# ghost ワンショット設計書

## 0. 目的

|              | 内容                                                                                          |
|--------------|-----------------------------------------------------------------------------------------------|
| **Goal**     | デーモンレスでシンプルな任意コマンドのバックグラウンド実行・管理ツールを Rust で提供する。 |
| **Non-Goal** | 常駐デーモン、リアルタイム通信、複雑なIPC。                                                   |

---

## 1. 全体アーキテクチャ

```
┌────────────┐                    ┌────────────┐
│ ghost CLI  │ ──── spawn ──────▶ │ background │
│ (clap)     │                    │  process   │
└────────────┘                    └────────────┘
     │                                    │
     │ write                        write │
     ▼                                    ▼
┌────────────┐                    ┌────────────┐
│ SQLite DB  │                    │ Log Files  │
│ (tasks.db) │                    │ (.ghost/)  │
├────────────┤                    ├────────────┤
│ Task table │                    │ Named Pipe │
│ PID info   │                    │ or RegFile │
│ Status     │                    └────────────┘
└────────────┘                            │
     ▲                                    │
     │ read/write                    read │
     │                                    ▼
     └──── ghost ui (ratatui) <───────────┘
```

**核心設計:**
- **No Daemon** — すべてのコマンドがワンショット実行
- **SQLite 状態管理** — 排他制御によるタスク情報の永続化
- **ファイルベースログ** — プロセス出力のファイル保存
- **孤児プロセス管理** — 意図的な親子関係の切断とプロセスグループ制御

---

## 2. クレート構成

| crate        | type | 役割                                                                |
|--------------|------|---------------------------------------------------------------------|
| `ghost-cli`  | bin  | `run / log / stop / kill / list / status`。状態管理とプロセス制御。 |
| `ghost-ui`   | bin  | ratatui TUI。DB polling + ログファイル監視。                        |
| `ghost-core` | lib  | 共通ロジック：DB操作、プロセス管理、ログ処理。                      |

---

## 3. 状態管理

### 3.1 SQLite スキーマ

```sql
CREATE TABLE tasks (
  id TEXT PRIMARY KEY,              -- UUID
  pid INTEGER NOT NULL,             -- プロセスID  
  pgid INTEGER,                     -- プロセスグループID
  command TEXT NOT NULL,            -- 実行コマンド (JSON array)
  env TEXT,                         -- 環境変数 (JSON object)
  cwd TEXT,                         -- 作業ディレクトリ
  status TEXT NOT NULL,             -- running/exited/killed
  exit_code INTEGER,                -- 終了コード
  started_at INTEGER NOT NULL,      -- 開始時刻 (Unix timestamp)
  finished_at INTEGER,              -- 終了時刻
  log_path TEXT NOT NULL            -- ログファイルパス
);

-- インデックス
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_pid ON tasks(pid);
```

### 3.2 排他制御

```rust
// WAL mode でパフォーマンス向上
let conn = Connection::open("~/.local/share/ghost/tasks.db")?;
conn.execute("PRAGMA journal_mode=WAL", [])?;
conn.execute("PRAGMA synchronous=NORMAL", [])?;

// 排他ロック
conn.execute("BEGIN EXCLUSIVE", [])?;
// ... CRUD operations ...
conn.execute("COMMIT", [])?;
```

---

## 4. プロセス管理

### 4.1 プロセス起動

```rust
fn spawn_background_process(cmd: &[String], env: &[(String, String)], cwd: &Path) -> Result<ProcessInfo> {
    let mut command = tokio::process::Command::new(&cmd[0]);
    command.args(&cmd[1..])
           .envs(env.iter().cloned())
           .current_dir(cwd)
           .stdin(Stdio::null())
           .stdout(Stdio::piped())
           .stderr(Stdio::piped());

    // 新しいプロセスグループを作成 (Unix)
    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            nix::unistd::setsid().map(|_| ()).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, e)
            })
        });
    }

    let mut child = command.spawn()?;
    let pid = child.id().unwrap();
    let pgid = get_process_group_id(pid)?;
    
    // ログファイル作成
    let log_path = create_log_file(&task_id)?;
    
    // stdout/stderr を非同期でログファイルに書き込み
    tokio::spawn(async move {
        copy_output_to_file(child.stdout.take(), child.stderr.take(), log_path).await
    });
    
    Ok(ProcessInfo { pid, pgid, ... })
}
```

### 4.2 プロセス停止

```rust
fn stop_process(task_id: &str, force: bool) -> Result<()> {
    let task = get_task_from_db(task_id)?;
    
    if force {
        // SIGKILL
        kill_process_group(task.pgid, Signal::SIGKILL)?;
    } else {
        // SIGTERM -> wait -> SIGKILL
        kill_process_group(task.pgid, Signal::SIGTERM)?;
        
        // グレースフル待機
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(10)).await;
            if process_exists(task.pid) {
                let _ = kill_process_group(task.pgid, Signal::SIGKILL);
            }
        });
    }
    
    update_task_status(task_id, "killed")?;
    Ok(())
}
```

---

## 5. ログ管理

### 5.1 ログファイル構成

```
~/.local/share/ghost/
├── tasks.db
└── logs/
    ├── {task-uuid-1}.log
    ├── {task-uuid-2}.log
    └── {task-uuid-3}.log
```

### 5.2 ログストリーミング実装

**Option A: tail -f 相当**
```rust
async fn follow_log(log_path: &Path) -> Result<impl Stream<Item = String>> {
    let file = File::open(log_path).await?;
    let reader = BufReader::new(file);
    let lines = reader.lines();
    
    // ファイル終端に達したら inotify で新しい行を待機
    Ok(lines.chain(watch_file_changes(log_path)))
}
```

**Option B: Named Pipe**
```rust
// プロセス起動時
let pipe_path = format!("/tmp/ghost-{}", task_id);
Command::new("mkfifo").arg(&pipe_path).output()?;

// プロセスの stdout/stderr を pipe にリダイレクト
command.stdout(Stdio::from(File::create(&pipe_path)?));

// ログ読み取り
async fn read_log_pipe(pipe_path: &str) -> Result<impl Stream<Item = String>> {
    let file = File::open(pipe_path).await?;
    let reader = BufReader::new(file);
    Ok(reader.lines())
}
```

---

## 6. CLI コマンド実装

### 6.1 `ghost run`

```rust
async fn cmd_run(args: RunArgs) -> Result<()> {
    let task_id = Uuid::new_v4().to_string();
    let log_path = get_log_path(&task_id);
    
    // DB にタスク記録
    insert_task(&task_id, &args.command, "running", &log_path)?;
    
    // バックグラウンドプロセス起動
    let process_info = spawn_background_process(&args.command, &args.env, &args.cwd)?;
    
    // DB を PID で更新
    update_task_pid(&task_id, process_info.pid, process_info.pgid)?;
    
    println!("{}", task_id);
    Ok(())
}
```

### 6.2 `ghost list`

```rust
async fn cmd_list() -> Result<()> {
    // DB から全タスク取得
    let mut tasks = get_all_tasks()?;
    
    // プロセス生存確認 & 状態更新
    for task in &mut tasks {
        if task.status == "running" && !process_exists(task.pid) {
            let exit_code = get_process_exit_code(task.pid);
            update_task_status(&task.id, "exited", exit_code)?;
            task.status = "exited".to_string();
            task.exit_code = exit_code;
        }
    }
    
    // 表示
    print_task_table(&tasks);
    Ok(())
}
```

### 6.3 `ghost log`

```rust
async fn cmd_log(task_id: &str, follow: bool) -> Result<()> {
    let task = get_task_from_db(task_id)?;
    let log_path = Path::new(&task.log_path);
    
    if follow {
        let mut stream = follow_log(log_path).await?;
        while let Some(line) = stream.next().await {
            println!("{}", line?);
        }
    } else {
        let content = tokio::fs::read_to_string(log_path).await?;
        print!("{}", content);
    }
    
    Ok(())
}
```

---

## 7. TUI 実装 (`ghost ui`)

### 7.1 アーキテクチャ

```rust
struct AppState {
    tasks: Vec<TaskInfo>,
    selected_task: Option<usize>,
    log_buffer: HashMap<String, VecDeque<String>>,
    show_logs: bool,
}

async fn run_tui() -> Result<()> {
    let mut app = AppState::new();
    let mut terminal = setup_terminal()?;
    
    loop {
        // DB から最新タスク一覧を取得
        app.refresh_tasks().await?;
        
        // 選択されたタスクのログを読み込み
        if let Some(task_id) = app.get_selected_task_id() {
            app.refresh_logs(task_id).await?;
        }
        
        // UI 描画
        terminal.draw(|f| ui::draw(f, &app))?;
        
        // イベント処理
        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => handle_key_event(&mut app, key).await?,
                _ => {}
            }
        }
    }
}
```

### 7.2 リアルタイム更新

```rust
impl AppState {
    async fn refresh_tasks(&mut self) -> Result<()> {
        // 定期的にDBポーリング
        self.tasks = get_all_tasks_with_status_update()?;
    }
    
    async fn refresh_logs(&mut self, task_id: &str) -> Result<()> {
        let task = self.get_task(task_id);
        let log_path = &task.log_path;
        
        // ログファイルの新しい行を読み込み
        let new_lines = read_new_log_lines(log_path, self.last_read_position)?;
        self.log_buffer.entry(task_id.to_string())
                       .or_default()
                       .extend(new_lines);
    }
}
```

---

## 8. クロスプラットフォーム対応

| 項目       | Linux / macOS              | Windows                    |
|------------|----------------------------|----------------------------|
| DB パス    | `~/.local/share/ghost/`    | `%APPDATA%\ghost\`         |
| プロセス制御 | `setsid()` + `killpg()`   | Job Object                 |
| シグナル   | SIGTERM/SIGKILL            | `TerminateProcess`         |
| ログパイプ | named pipe (mkfifo)        | Named pipe (CreateNamedPipe) |

---

## 9. トレードオフ分析

### 9.1 ワンショット設計の利点

- **シンプルさ**: デーモン管理不要、IPC 実装不要
- **軽量性**: 常駐プロセスなし、メモリ使用量最小
- **デバッグ容易**: 単一プロセスでのデバッグ
- **デプロイ簡単**: バイナリ配置のみ

### 9.2 ワンショット設計の課題

- **パフォーマンス**: DB アクセスとプロセス監視のオーバーヘッド
- **リアルタイム性**: ポーリングベースの状態更新
- **競合制御**: 複数コマンド同時実行時のDB競合
- **ログ配信**: リアルタイムログストリーミングの複雑さ

---

## 10. 実装優先度

### Phase 1: MVP
- [x] 基本的なプロセス起動・停止
- [x] SQLite による状態管理
- [x] ファイルベースログ保存
- [ ] `ghost run/stop/list/log` コマンド

### Phase 2: ログ機能強化
- [ ] `ghost log --follow` 実装
- [ ] ログローテーション
- [ ] 複数リーダー対応

### Phase 3: TUI
- [ ] 基本的な TUI インターフェース
- [ ] リアルタイムログ表示
- [ ] キーボードショートカット

### Phase 4: 最適化・安定性
- [ ] DB 接続プーリング
- [ ] プロセス監視の効率化
- [ ] エラーハンドリング強化
- [ ] Windows 対応

---

## 11. 技術的実装詳細

### 11.1 プロセス生存確認

```rust
fn process_exists(pid: u32) -> bool {
    #[cfg(unix)]
    {
        nix::sys::signal::kill(
            nix::unistd::Pid::from_raw(pid as i32), 
            None
        ).is_ok()
    }
    
    #[cfg(windows)]
    {
        // OpenProcess + GetExitCodeProcess
        // TODO: Windows implementation
        true
    }
}
```

### 11.2 ログファイル監視

```rust
async fn watch_log_file(path: &Path) -> Result<impl Stream<Item = String>> {
    use tokio::fs::File;
    use tokio::io::{AsyncBufReadExt, BufReader};
    
    let file = File::open(path).await?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    
    Ok(async_stream::stream! {
        loop {
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF reached, wait for new data
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
                Ok(_) => {
                    yield line.trim_end().to_string();
                    line.clear();
                }
                Err(_) => break,
            }
        }
    })
}
```

---

## 12. 参考コマンド例

```bash
# プロセス起動
ghost run sleep 100
# -> 出力: a1b2c3d4-e5f6-7890-abcd-ef1234567890

# 一覧表示  
ghost list
# -> ID       | Status  | Command   | Started
#    a1b2... | running | sleep 100 | 2025-06-30 10:30

# ログ確認
ghost log a1b2c3d4 --follow

# プロセス停止
ghost stop a1b2c3d4

# TUI 起動
ghost ui
```

---

この設計でよろしいでしょうか？特に気になる部分や追加で詳細化したい部分があれば教えてください。
