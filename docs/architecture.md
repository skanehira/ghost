# Ghost Architecture Document

## Overview

Ghost is a lightweight background process manager for Unix systems (Linux, macOS, BSD) that follows a daemon-free, one-shot execution model. Unlike traditional process managers that require a persistent daemon, Ghost executes each command independently while maintaining state through SQLite.

## Core Design Principles

1. **No Daemon Required**: Every command runs as a standalone process
2. **SQLite-based State Management**: Persistent task information without a daemon
3. **File-based Logging**: Simple, reliable log storage
4. **Process Group Isolation**: Proper signal handling and cleanup
5. **Async I/O**: Non-blocking operations using Tokio

## Architecture

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
│ (tasks.db) │                    │ (*.log)    │
├────────────┤                    └────────────┘
│ Task table │                           │
│ PID info   │                      read │
│ Status     │                           ▼
└────────────┘                    ┌────────────┐
     ▲                            │   ghost    │
     │ read/update                │    TUI     │
     └────────────────────────────│ (ratatui)  │
                                  └────────────┘
```

## Component Structure

### Binary Crates

| Component      | Purpose                                                                    |
|----------------|----------------------------------------------------------------------------|
| `ghost` (main) | CLI interface with subcommands: run, list, log, stop, status, cleanup, tui |

### Library Modules

| Module          | Purpose                                        |
|-----------------|------------------------------------------------|
| `app::commands` | Command implementations (run, list, log, etc.) |
| `app::storage`  | SQLite database operations and task management |
| `app::process`  | Process spawning and management                |
| `app::tui`      | Terminal UI implementation using ratatui       |
| `app::config`   | Configuration and path management              |
| `app::error`    | Error types and handling                       |

## Database Schema

```sql
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    pid INTEGER NOT NULL,
    pgid INTEGER,
    command TEXT NOT NULL,
    env TEXT,
    cwd TEXT,
    status TEXT NOT NULL DEFAULT 'running',
    exit_code INTEGER,
    started_at INTEGER NOT NULL,
    finished_at INTEGER,
    log_path TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_pid ON tasks(pid);
CREATE INDEX IF NOT EXISTS idx_tasks_started_at ON tasks(started_at);
```

### Task Status Values

- `running`: Process is currently active
- `exited`: Process terminated normally
- `killed`: Process was killed by signal
- `unknown`: Process state cannot be determined

## Process Management

### Process Spawning

1. Generate UUID for task identification
2. Create new process group using `setsid()` 
3. Redirect stdout/stderr to log file
4. Store task metadata in SQLite
5. Return task ID to user

```rust
// Simplified process spawning flow
let task_id = Uuid::new_v4().to_string();
let mut child = Command::new(&command[0])
    .args(&command[1..])
    .stdin(Stdio::null())
    .stdout(log_file.try_clone()?)
    .stderr(log_file)
    .spawn()?;

// Create new process group
unsafe {
    libc::setpgid(child.id() as i32, 0);
}
```

### Process Lifecycle

1. **Start**: Process spawned with redirected I/O
2. **Monitor**: Status checked via signal 0 when queried
3. **Stop**: SIGTERM sent (graceful), or SIGKILL (forced)
4. **Cleanup**: Database updated, logs preserved

### Signal Handling

- Uses process groups for clean subprocess termination
- SIGTERM for graceful shutdown
- SIGKILL for forced termination
- Signal 0 for process existence check

## Log Management

### Log Storage

```
$GHOST_DATA_DIR/
├── tasks.db
└── logs/
    ├── {task-uuid-1}.log
    ├── {task-uuid-2}.log
    └── {task-uuid-3}.log
```

### Log Features

- Combined stdout/stderr capture
- Real-time following via file polling
- Automatic cleanup with task deletion
- No rotation (kept simple by design)

### MCP Server Logging

When running in MCP server mode (`ghost mcp`), Ghost outputs diagnostic logs to a separate file for debugging connection issues:

```
$GHOST_DATA_DIR/logs/ghost.log
```

**Platform-specific locations:**
- Linux: `~/.local/share/ghost/logs/ghost.log`
- macOS: `~/Library/Application Support/ghost/logs/ghost.log`

**Log rotation:**
- Size-based rotation at 5MB
- Keeps up to 5 historical files (`ghost.log.1`, `ghost.log.2`, etc.)
- Uses non-blocking async writes for performance

**Logged events:**
- Server startup
- Initialization completion
- Transport/connection errors
- Server shutdown

This logging is essential for debugging MCP protocol issues since the MCP server uses stdio for communication, making console output unavailable.

## TUI Architecture

### Components

1. **Task List View**: Displays all tasks with filtering
2. **Log Viewer**: Scrollable log display with memory limits
3. **Event Loop**: Non-blocking key handling with EventStream

### Key Features

- Real-time status updates (1-second refresh)
- Memory-efficient log viewing (10k line limit)
- Horizontal/vertical scrolling
- Task filtering (All/Running/Exited/Killed)
- Process termination (SIGTERM/SIGKILL)

### TUI Keybindings

**Task List:**
- `j`/`k`: Move selection up/down
- `g`/`G`: Jump to top/bottom
- `l`: View logs for selected task
- `s`: Send SIGTERM to selected task
- `Ctrl+K`: Send SIGKILL to selected task
- `Tab`: Cycle through filters
- `q`: Quit

**Log Viewer:**
- `j`/`k`: Scroll up/down
- `h`/`l`: Scroll left/right
- `g`/`G`: Jump to top/bottom
- `Esc`: Return to task list

## Configuration

### Environment Variables

- `GHOST_DATA_DIR`: Override default data directory location

### Default Paths

When `GHOST_DATA_DIR` is not set, Ghost uses platform-specific default locations:

**Linux:**
- Data: `$XDG_DATA_HOME/ghost` or `~/.local/share/ghost`
- Logs: `$XDG_DATA_HOME/ghost/logs` or `~/.local/share/ghost/logs`

**macOS:**
- Data: `~/Library/Application Support/ghost/`
- Logs: `~/Library/Application Support/ghost/logs/`

The data directory contains:
- `tasks.db`: SQLite database with task metadata
- `logs/`: Directory containing log files for each task

## Error Handling

### Error Types

```rust
pub enum GhostError {
    Io { source: std::io::Error },
    Database { source: rusqlite::Error },
    InvalidArgument { message: String },
    TaskNotFound { task_id: String },
    TaskNotRunning { task_id: String, status: String },
    // ...
}
```

### Error Strategy

- Explicit error types for different failure modes
- User-friendly error messages
- Non-zero exit codes for CLI failures
- TUI shows errors without crashing

## Performance Considerations

### Optimizations

1. **WAL Mode**: SQLite uses Write-Ahead Logging for concurrency
2. **Lazy Loading**: TUI loads logs incrementally
3. **Memory Limits**: Log viewer caps at 10k lines
4. **Efficient Queries**: Indexed by status and timestamp

### Trade-offs

- **Simplicity over Performance**: One-shot model has overhead
- **Reliability over Speed**: Synchronous DB writes ensure consistency
- **Portability over Features**: Unix-only for cleaner implementation

## Security

- SQLite database permissions: 0644 (relies on directory permissions)
- Log files: Inherit directory permissions
- No authentication (single-user design)
- Process isolation via separate process groups

## Testing Strategy

### Unit Tests
- Database operations
- Process state management
- Configuration handling
- TUI components

### Integration Tests
- End-to-end command flows
- Process lifecycle management
- Signal handling

### E2E Tests
- Full command-line workflows
- Multi-process scenarios
- Error conditions

## Future Considerations

### Potential Enhancements

1. **Resource Limits**: CPU/memory constraints via cgroups
2. **Task Dependencies**: Sequential/parallel task execution
3. **Remote API**: REST/gRPC interface
4. **Log Rotation**: Size/time-based rotation
5. **Task Scheduling**: Cron-like functionality

### Explicitly Out of Scope

1. **Multi-user Support**: Single-user by design
2. **Distributed Execution**: Local only
3. **Windows Support**: Unix-specific process management
4. **Complex Orchestration**: Not a container orchestrator

## Maintenance

### Database Maintenance

- `cleanup` command removes old tasks
- 30-day default retention
- Manual cleanup supported via `--all` flag

### Log Maintenance

- Logs deleted with tasks
- No automatic rotation
- Manual cleanup via task deletion

## Development Workflow

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
cargo nextest run  # Better test output
./e2e/run_all_tests.sh  # E2E tests
```

### Code Quality

```bash
cargo fmt
cargo clippy -- -D warnings
```

## Conclusion

Ghost achieves its goal of simple, reliable background process management through:

1. Daemon-free architecture for simplicity
2. SQLite for persistent state without IPC complexity
3. File-based logging for reliability
4. Process groups for proper cleanup
5. Modern Rust practices for safety and performance

The design prioritizes simplicity, reliability, and ease of use over advanced features, making it ideal for development environments and simple automation tasks.
