# Ghost Project TODO

## TUI (Terminal User Interface) 実装

### Phase 1: 基盤構築 ✅
- [x] **TUIライブラリの選定と導入**
  - ratatui (crossterm backend) の依存関係追加
  - 基本的なTUIアプリケーション構造の実装
  - イベントループとキーボード入力処理

- [x] **基本レイアウト設計**
  - メインビュー: タスクリスト表示領域
  - ヘッダー: アプリケーション情報とフィルター表示
  - フッター: キーバインド表示

### Phase 2: タスクリスト表示 ✅
- [x] **タスクリスト表示機能**
  - データベースからタスク一覧を取得して表示
  - 表形式でのタスク情報表示 (ID, PID, Status, Started, Command)
  - スクロール機能 (上下キーでの移動)
  - リアルタイム更新対応

- [x] **ステータスフィルタリング**
  - フィルター状態の視覚的な表示
  - フィルター変更時の表示更新

### Phase 3: 基本操作機能 ✅
- [x] **タスク選択とナビゲーション**
  - 上下キーでタスク選択
  - 選択状態の視覚的ハイライト
  - ページアップ/ダウン機能

- [ ] **基本操作の実装**
  - `Enter`: 選択タスクの詳細表示
  - `s`: タスク停止 (stop)
  - `k`: タスク強制終了 (kill)
  - [x] `l`: ログ表示 ✅
  - `r`: 新規タスク実行

### Phase 4: ログ表示機能 ✅
- [x] **ログビューアーの実装**
  - 選択タスクのログ内容表示
  - スクロール可能なログ表示
  - ログビューからメインビューへの復帰 (Escキー)
  - 基本的なキーバインド (j/k, gg/G)

- [ ] **ログ表示の最適化**
  - ログの動的更新 (tail -f 相当)
  - 大容量ログファイルの効率的表示
  - 検索機能 (/ キーで検索モード)
  - 行番号表示オプション

### Phase 5: タスク実行・管理
- [ ] **新規タスク実行機能 (削除済み)**
  - TUIでの新規タスク実行は不要
  - CLIの `ghost run` コマンドを使用

- [ ] **タスク操作の確認ダイアログ**
  - 停止/削除前の確認ダイアログ
  - `y/n` での操作確認
  - 操作結果の通知表示

### Phase 6: 高度な機能
- [ ] **詳細情報表示**
  - タスクの詳細情報ポップアップ
  - プロセス情報 (PID, PGID, 実行時間等)
  - 環境変数とコマンドライン引数表示

- [ ] **設定とカスタマイズ**
  - キーバインドのカスタマイズ
  - 表示カラムの選択
  - 更新間隔の設定
  - テーマ/カラー設定

- [ ] **検索とフィルタリング**
  - コマンド名での検索
  - タスクIDでの検索
  - 実行時間での範囲フィルタ
  - 正規表現対応

### Phase 7: クリーンアップ・保守性
- [ ] **エラーハンドリング強化**
  - TUI操作中のエラー表示
  - ネットワーク/DB接続エラーの処理
  - 優雅な終了処理

- [ ] **テスト実装**
  - TUIコンポーネントの単体テスト
  - キーボード操作のテスト
  - 表示ロジックのテスト

- [ ] **ドキュメント整備**
  - TUI操作方法のドキュメント
  - キーバインド一覧
  - 設定ファイル仕様

## 技術仕様詳細

### 使用ライブラリ
- **ratatui**: TUIフレームワーク (crossterm backend推奨)
- **crossterm**: クロスプラットフォーム端末制御
- **tokio**: 非同期実行時用 (既存)

### キーバインド設計 (Vim-like)
```
# 基本移動 (Vim準拠)
j/↓         : 下に移動
k/↑         : 上に移動
gg          : 最上部に移動
G           : 最下部に移動
Ctrl+f      : ページダウン
Ctrl+b      : ページアップ
Ctrl+d      : 半ページダウン
Ctrl+u      : 半ページアップ

# 表示モード切り替え
Tab         : フィルター切り替え (All → Running → Exited → Killed)
1           : All表示
2           : Running表示
3           : Exited表示
4           : Killed表示

# タスク操作 (Vim-like)
Enter/o     : タスク詳細表示
l           : ログ表示 (view log)
s           : タスク停止 (stop/SIGTERM)
K           : タスク強制終了 (kill/SIGKILL)
dd          : タスク削除 (delete)
cc          : クリーンアップ (cleanup)

# 検索・フィルタリング (Vim準拠)
/           : 検索モード開始
n           : 次の検索結果
N           : 前の検索結果
:           : コマンドモード (将来拡張用)

# システム操作
R           : 手動更新 (refresh)
h/?         : ヘルプ表示
q           : 終了
Ctrl+C      : 強制終了
Esc         : キャンセル/戻る/通常モードへ

# 選択・マーク (将来拡張)
v           : ビジュアルモード開始
V           : 行選択モード
Space       : タスクマーク/アンマーク
```

### 画面レイアウト設計

#### メイン画面構成
```
┌─────────────────────────────────────────────────────────────────────┐
│ Ghost TUI v1.0.0                                    [Filter: All] │ ← ヘッダー
├─────────────────────────────────────────────────────────────────────┤
│ ID       │ PID    │ Status  │ Started           │ Command          │ ← テーブルヘッダー
├─────────┼────────┼─────────┼───────────────────┼──────────────────┤
│ abc12345 │ 12345  │ Running │ 2024-01-01 10:00 │ npm run dev      │ ← 選択行(ハイライト)
│ def67890 │ 67890  │ Exited  │ 2024-01-01 09:30 │ cargo build      │
│ ghi11111 │ 11111  │ Killed  │ 2024-01-01 09:00 │ python script.py │
│          │        │         │                   │                  │
│          │        │         │                   │                  │ ← 空行領域
│          │        │         │                   │                  │
├─────────────────────────────────────────────────────────────────────┤
│ j/k:Move  Enter:Detail  l:Log  s:Stop  K:Kill  dd:Delete  q:Quit       │ ← フッター
└─────────────────────────────────────────────────────────────────────┘
```

#### 詳細表示モード
```
┌─────────────────────────────────────────────────────────────────────┐
│ Task Detail: abc12345                                     [Esc:Back] │
├─────────────────────────────────────────────────────────────────────┤
│ Task ID    : abc12345-6789-abcd-ef01-234567890123                   │
│ PID        : 12345                                                  │
│ PGID       : 12345                                                  │
│ Status     : Running                                                │
│ Exit Code  : -                                                      │
│ Started    : 2024-01-01 10:00:15                                    │
│ Finished   : -                                                      │
│ Command    : ["npm", "run", "dev"]                                  │
│ CWD        : /home/user/project                                     │
│ Log Path   : /home/user/.local/share/ghost/logs/abc12345.log        │
│                                                                     │
│ Environment Variables:                                              │
│   NODE_ENV=development                                              │
│   PATH=/usr/local/bin:/usr/bin:/bin                                 │
│                                                                     │
├─────────────────────────────────────────────────────────────────────┤
│ l:ViewLog  s:Stop  K:Kill  dd:Delete  Esc:Back                      │
└─────────────────────────────────────────────────────────────────────┘
```

#### ログ表示モード
```
┌─────────────────────────────────────────────────────────────────────┐
│ Log View: abc12345 (npm run dev)                         [Esc:Back] │
├─────────────────────────────────────────────────────────────────────┤
│ Starting development server...                                      │
│ Webpack compilation started                                         │
│ ✓ Compiled successfully                                             │
│ Server running on http://localhost:3000                            │
│ File change detected: src/App.js                                   │
│ Recompiling...                                                      │
│ ✓ Compiled successfully                                             │ ← 現在位置
│ │                                                                   │ ← スクロール可能
│ │                                                                   │
│ │                                                                   │
│ │                                                                   │
├─────────────────────────────────────────────────────────────────────┤
│ j/k:Scroll  gg/G:Top/Bottom  /:Search  f:Follow  Esc:Back          │
└─────────────────────────────────────────────────────────────────────┘
```


#### 検索モード
```
┌─────────────────────────────────────────────────────────────────────┐
│ Ghost TUI v1.0.0                                    [Filter: All] │
├─────────────────────────────────────────────────────────────────────┤
│ ID       │ PID    │ Status  │ Started           │ Command          │
├─────────┼────────┼─────────┼───────────────────┼──────────────────┤
│ abc12345 │ 12345  │ Running │ 2024-01-01 10:00 │ npm run dev      │ ← マッチ (ハイライト)
│ def67890 │ 67890  │ Exited  │ 2024-01-01 09:30 │ cargo build      │
│ ghi11111 │ 11111  │ Killed  │ 2024-01-01 09:00 │ python script.py │
│          │        │         │                   │                  │
│          │        │         │                   │                  │
│          │        │         │                   │                  │
├─────────────────────────────────────────────────────────────────────┤
│ Search: npm█                                    [2 matches found] │ ← 検索入力
└─────────────────────────────────────────────────────────────────────┘
```

### 色・テーマ設計

#### ステータス別カラー
- **Running**: 緑 (Green)
- **Exited**: 青 (Blue) - 正常終了
- **Killed**: 赤 (Red) - 異常終了
- **選択行**: 黄色背景 (Yellow Background)
- **ヘッダー**: 白背景/黒文字 (White/Black)

#### UI要素カラー
- **枠線**: グレー (Gray)
- **アクティブフィールド**: シアン (Cyan)
- **エラーメッセージ**: 赤 (Red)
- **成功メッセージ**: 緑 (Green)
- **検索マッチ**: マゼンタ背景 (Magenta Background)

### レスポンシブ設計

#### 最小サイズ対応
- 最小幅: 80文字
- 最小高: 24行
- 幅が狭い場合: Command列を短縮表示
- 高さが低い場合: フッターの一部省略

#### 可変カラム幅
- ID: 8文字 (固定)
- PID: 8文字 (固定)
- Status: 8文字 (固定)
- Started: 17文字 (固定)
- Command: 残り全幅 (可変)

### アーキテクチャ方針
- 既存のCLIコマンドロジックを再利用
- TUIとCLIの共存 (サブコマンドとして `ghost tui`)
- 状態管理は最小限に (データベースが真の状態)
- エラーハンドリングは既存のGhostError活用

### 実装優先度
1. **High**: Phase 1-3 (基盤とリスト表示)
2. **Medium**: Phase 4-5 (ログとタスク管理)
3. **Low**: Phase 6-7 (高度な機能と保守性)

## パフォーマンス最適化 (将来実装予定)

### データベースクエリの最適化
- [ ] **インデックス追加でクエリ効率化**
  - tasks テーブルの status カラムにインデックス追加
  - finished_at カラムにインデックス追加 (cleanup処理用)
  - 複合インデックスの検討 (status + finished_at)

- [ ] **現在のクエリパフォーマンス調査**
  - SQLiteのEXPLAIN QUERY PLANでボトルネック特定
  - 大量タスク環境でのパフォーマンステスト
  - クエリ実行時間の計測とログ出力

### ログファイル読み込みの効率化
- [ ] **大きなログファイルの処理改善**
  - ストリーミング読み込みの実装
  - メモリ使用量を抑えた tail -n 相当の機能
  - ログローテーション対応

- [ ] **ファイル監視の最適化**
  - notify クレートの設定チューニング
  - ファイル変更検知の効率化
  - 複数ファイル同時監視時のリソース使用量最適化

### その他の最適化候補
- [ ] **メモリ使用量の最適化**
  - タスクリスト表示時のメモリ効率化
  - 長時間実行時のメモリリーク調査

- [ ] **並行処理の改善**
  - 複数コマンド同時実行時のパフォーマンス
  - データベース接続プールの検討

- [ ] **起動時間の最適化**
  - 初回起動時のデータベース初期化時間短縮
  - 設定ファイル読み込み最適化

## 実装優先度
1. **High**: データベースインデックス追加 (即効性あり)
2. **Medium**: ログファイル読み込み効率化 (ユーザー体験向上)  
3. **Low**: その他の最適化 (測定後に判断)

## 注意事項
- パフォーマンス最適化前に必ずベンチマークを取る
- 最適化後は E2E テストで機能回帰がないことを確認
- 大量データでのテスト環境を準備してから実施