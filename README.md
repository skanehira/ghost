![GitHub Repo stars](https://img.shields.io/github/stars/skanehira/ghost?style=social)
![GitHub](https://img.shields.io/github/license/skanehira/ghost)
![GitHub all releases](https://img.shields.io/github/downloads/skanehira/ghost/total)
![GitHub CI Status](https://img.shields.io/github/actions/workflow/status/skanehira/ghost/ci.yaml?branch=main)
![GitHub Release Status](https://img.shields.io/github/v/release/skanehira/ghost)

# Ghost

Ghost is a simple background process manager for Unix systems (Linux, macOS, BSD).

## Features

- **Background Process Execution**: Run commands in the background without a daemon
- **TUI Mode**: Interactive terminal UI for managing processes
- **Process Management**: Start, stop, and monitor processes
- **Log Management**: Automatic log file creation and real-time following
- **Working Directory Tracking**: See where each command was executed
- **No Daemon Required**: Simple one-shot execution model

This tool was inspired by:
- [pueue](https://github.com/Nukesor/pueue)
- [task-spooler](https://github.com/justanhduc/task-spooler)

## Installation

### Requirements

- Unix-based system (Linux, macOS, BSD)
- Rust 1.80+ (2024 edition)

### Build from source

```bash
git clone https://github.com/skanehira/ghost.git
cd ghost
cargo build --release
```

The binary will be available at `target/release/ghost`.

### Install

```bash
# Copy to local bin directory
cp target/release/ghost ~/.local/bin/

# Or to system bin (requires sudo)
sudo cp target/release/ghost /usr/local/bin/
```

## Usage

Ghost is a command-line tool for managing background processes without requiring a daemon. All commands work with a simple one-shot execution model.

### Basic Commands

#### Run a command in the background

```bash
# Run a simple command
$ ghost run echo "Hello, World"
Started background process:
  Task ID: 83efed6c-ae7d-4c26-8993-5d2d1a83b64f
  PID: 11203
  Log file: /Users/user/Library/Application Support/ghost/logs/83efed6c-ae7d-4c26-8993-5d2d1a83b64f.log

# Run a long-running command
$ ghost run sleep 30
Started background process:
  Task ID: e56ed5f8-44c8-4905-97aa-651164afd37e
  PID: 8969
  Log file: /Users/user/Library/Application Support/ghost/logs/e56ed5f8-44c8-4905-97aa-651164afd37e.log

# Run a script
$ ghost run ./my_script.sh
Started background process:
  Task ID: c3d4e5f6-g7h8-9012-cdef-345678901234
  PID: 12347
  Log file: /Users/user/Library/Application Support/ghost/logs/c3d4e5f6-g7h8-9012-cdef-345678901234.log

# Run a command with specific working directory
$ ghost run --cwd /path/to/directory make build

# Run with environment variables
$ ghost run --env NODE_ENV=production --env PORT=3000 npm start
```

#### List all tasks

```bash
# List all tasks
$ ghost list
Task ID                              PID      Status     Started              Command                        Directory
--------------------------------------------------------------------------------------------------------------------------------------
e56ed5f8-44c8-4905-97aa-651164afd37e 8969     exited     2025-07-01 15:36     sleep 30                       /home/user/projects
9fe034eb-2ce7-4809-af10-2c99af15583d 8730     exited     2025-07-01 15:35     echo Hello, World              /home/user
cb5b7275-1234-5678-abcd-ef0123456789 8349     running    2025-07-01 15:34     npm run dev                    /home/user/my-app
98765f79-abcd-ef01-2345-6789abcdef01 5404     killed     2025-07-01 15:29     ./scripts/hello_loop.sh        /home/user/scripts
89afd966-5678-90ab-cdef-1234567890ab 5298     exited     2025-07-01 15:29     ./scripts/hello_loop.sh        /home/user/scripts       

# Filter by status
$ ghost list --status running       
```

#### View task logs

```bash
# View logs for a specific task
$ ghost log 9fe034eb-2ce7-4809-af10-2c99af15583d
Hello, World

# View logs for a script with multiple outputs
$ ghost log c3d4e5f6-g7h8-9012-cdef-345678901234
Starting script execution...
Processing file 1/10
Processing file 2/10
...
Script completed successfully

# Follow logs in real-time (like tail -f)
$ ghost log e56ed5f8-44c8-4905-97aa-651164afd37e --follow
Following logs for task e56ed5f8-44c8-4905-97aa-651164afd37e (Ctrl+C to stop):
Log file: /Users/user/Library/Application Support/ghost/logs/e56ed5f8-44c8-4905-97aa-651164afd37e.log
----------------------------------------
[Follow mode not fully implemented yet]
```

#### Check task status

```bash
# Get detailed information about a running task
$ ghost status e56ed5f8-44c8-4905-97aa-651164afd37e
Task: e56ed5f8-44c8-4905-97aa-651164afd37e
PID: 8969
Status: running
Command: sleep 30
Working directory: /home/user/projects
Started: 2025-07-01 15:36:23
Log file: /Users/user/Library/Application Support/ghost/logs/e56ed5f8-44c8-4905-97aa-651164afd37e.log

# Get detailed information about a completed task
$ ghost status 9fe034eb-2ce7-4809-af10-2c99af15583d
Task: 9fe034eb-2ce7-4809-af10-2c99af15583d
PID: 8730
Status: exited
Command: echo Hello, World
Working directory: /home/user
Started: 2025-07-01 15:35:36
Finished: 2025-07-01 15:36:10
Exit code: 0
Log file: /Users/user/Library/Application Support/ghost/logs/9fe034eb-2ce7-4809-af10-2c99af15583d.log
```

#### Stop a running task

```bash
# Gracefully stop a task (SIGTERM)
$ ghost stop e56ed5f8-44c8-4905-97aa-651164afd37e
Process e56ed5f8-44c8-4905-97aa-651164afd37e (8969) has been exited

# Force kill a task (SIGKILL)
$ ghost stop c3d4e5f6-g7h8-9012-cdef-345678901234 --force
Process c3d4e5f6-g7h8-9012-cdef-345678901234 (12347) has been killed

# Try to stop an already stopped task
$ ghost stop 9fe034eb-2ce7-4809-af10-2c99af15583d
Error: Task 9fe034eb is not running (status: exited)
```

#### Clean up old tasks

```bash
# Clean up tasks older than 30 days (default)
$ ghost cleanup
Successfully deleted 5 task(s).
Deleted tasks older than 30 days with status: exited, killed.

# Clean up tasks older than 7 days
$ ghost cleanup --days 7

# Preview what would be deleted (dry run)
$ ghost cleanup --dry-run
The following 3 task(s) would be deleted:
Task ID                              PID      Status     Started              Command                        Directory
--------------------------------------------------------------------------------------------------------------------------------------
89afd966-5678-90ab-cdef-1234567890ab 5298     exited     2025-06-01 15:29     ./scripts/backup.sh            /home/user
6a279d57-1234-5678-abcd-ef0123456789 5203     exited     2025-06-01 15:29     ./scripts/cleanup.sh           /home/user

# Clean up all finished tasks regardless of age
$ ghost cleanup --all

# Clean up only killed tasks
$ ghost cleanup --status killed
```

### TUI Mode

Ghost includes an interactive Terminal User Interface for managing processes:

```bash
$ ghost tui
```

**TUI Features:**
- Real-time process status updates (refreshes every second)
- Interactive task management
- Log viewer with line numbers
- Keyboard navigation

**TUI Keybindings:**
- `j`/`k` or `↓`/`↑`: Move selection up/down
- `g`/`G`: Jump to top/bottom of list
- `l` or `Enter`: View logs for selected task
- `s`: Send SIGTERM to selected task
- `Ctrl+K`: Send SIGKILL to selected task  
- `q` or `Esc`: Quit/Go back
- `Tab`: Switch between task filters (All/Running/Exited/Killed)

**Log Viewer Keybindings:**
- `j`/`k` or `↓`/`↑`: Scroll up/down
- `h`/`l` or `←`/`→`: Scroll left/right (for long lines)
- `g`/`G`: Jump to top/bottom
- `Page Up`/`Page Down`: Scroll by page
- `Esc` or `q`: Return to task list

### Examples

Here are some practical examples of using Ghost:

```bash
# Start a web server in the background
$ ghost run python -m http.server 8080
Started background process:
  Task ID: d4e5f6g7-h8i9-0123-defg-456789012345
  PID: 12348
  Log file: /Users/user/Library/Application Support/ghost/logs/d4e5f6g7-h8i9-0123-defg-456789012345.log

# Run tests while working on other things
$ ghost run npm test
Started background process:
  Task ID: e5f6g7h8-i9j0-1234-efgh-567890123456
  PID: 12349
  Log file: /Users/user/Library/Application Support/ghost/logs/e5f6g7h8-i9j0-1234-efgh-567890123456.log

# Start a long-running backup process
$ ghost run rsync -av /home/user/ /backup/
Started background process:
  Task ID: f6g7h8i9-j0k1-2345-fghi-678901234567
  PID: 12350
  Log file: /Users/user/Library/Application Support/ghost/logs/f6g7h8i9-j0k1-2345-fghi-678901234567.log

# Run a development server with environment variables
$ ghost run --env NODE_ENV=development --env PORT=3000 --cwd ~/my-app npm run dev
Started background process:
  Task ID: a1b2c3d4-e5f6-7890-abcd-ef1234567890
  PID: 12351
  Log file: /Users/user/Library/Application Support/ghost/logs/a1b2c3d4-e5f6-7890-abcd-ef1234567890.log

# Run multiple commands in parallel
$ ghost run --cwd ~/project1 npm run build
$ ghost run --cwd ~/project2 cargo build --release
$ ghost run --cwd ~/project3 go build

# Monitor all running tasks in TUI
$ ghost tui
```

### Task Management

Each background task gets:
- **Unique Task ID**: Used to reference the task in other commands
- **Process ID (PID)**: The actual system process ID
- **Log File**: Captures stdout and stderr output
- **Status Tracking**: Monitors running, exited, or killed states
- **Timestamps**: Records start and finish times

### Key Features

- **No Daemon Required**: Each command is self-contained
- **Process Isolation**: Tasks run as independent processes
- **Log Persistence**: All output is captured and stored
- **Status Monitoring**: Real-time status updates via process checking
- **Cross-Platform**: Works on Unix-like systems (Linux, macOS)

### Configuration

#### Environment Variables

- `GHOST_DATA_DIR`: Override data directory location (default: platform-specific)
- `GHOST_LOG_DIR`: Override log directory location (default: `$GHOST_DATA_DIR/logs`)

#### Default Locations

**Linux:**
- Data: `~/.local/share/ghost/`
- Logs: `~/.local/share/ghost/logs/`

**macOS:**
- Data: `~/Library/Application Support/ghost/`
- Logs: `~/Library/Application Support/ghost/logs/`

### Architecture

Ghost uses a simple, daemon-free architecture:

1. **SQLite Database**: Stores task metadata and status (`ghost.db`)
2. **File-based Logs**: Each task gets its own log file
3. **Process Groups**: Uses `setsid()` for proper signal handling and cleanup
4. **Status Monitoring**: Real-time process checking via signal 0
5. **Non-blocking TUI**: Uses async event streams for responsive UI

### Technical Details

- **Language**: Rust (2024 edition)
- **Database**: SQLite with bundled driver
- **TUI Framework**: Ratatui with tui-scrollview
- **Async Runtime**: Tokio
- **Process Management**: Unix signals (SIGTERM/SIGKILL)
- **Platform Support**: Unix-only (uses `nix` crate for system calls)
