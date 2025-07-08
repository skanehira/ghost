![GitHub Repo stars](https://img.shields.io/github/stars/skanehira/ghost?style=social)
![GitHub](https://img.shields.io/github/license/skanehira/ghost)
![GitHub all releases](https://img.shields.io/github/downloads/skanehira/ghost/total)
![GitHub CI Status](https://img.shields.io/github/actions/workflow/status/skanehira/ghost/ci.yaml?branch=main)
![GitHub Release Status](https://img.shields.io/github/v/release/skanehira/ghost)

# Ghost

> **注意**: このリポジトリは[skanehira/ghost](https://github.com/skanehira/ghost)の個人的改良版です。本家のキーバインド等だけ改良した版です。趣味嗜好の部分が大きいのでghostにPRもしません。私の方で長期メンテもないでしょうが、しばらく愛用します。

Ghost is a simple background process manager for Unix systems (Linux, macOS, BSD).

<div align="center">
  <img src="./images/logo.png" width=300 />
</div>

## Features

![](./images/ghost.png)

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
# View logs for a specific task (full UUID)
$ ghost log 9fe034eb-2ce7-4809-af10-2c99af15583d
Hello, World

# View logs using short ID (more convenient)
$ ghost log 9fe034eb
Hello, World

# View logs for a script with multiple outputs
$ ghost log c3d4e5f6
Starting script execution...
Processing file 1/10
Processing file 2/10
...
Script completed successfully

# Follow logs in real-time (like tail -f)
$ ghost log -f e56ed5f8
Following logs for task e56ed5f8-44c8-4905-97aa-651164afd37e (Ctrl+C to stop):
Log file: /Users/user/Library/Application Support/ghost/logs/e56ed5f8-44c8-4905-97aa-651164afd37e.log
----------------------------------------
Processing file 1/10
Processing file 2/10
...
```

#### Check task status

```bash
# Get detailed information about a running task (using short ID)
$ ghost status e56ed5f8
Task: e56ed5f8-44c8-4905-97aa-651164afd37e
PID: 8969
Status: running
Command: sleep 30
Working directory: /home/user/projects
Started: 2025-07-01 15:36:23
Log file: /Users/user/Library/Application Support/ghost/logs/e56ed5f8-44c8-4905-97aa-651164afd37e.log

# Get detailed information about a completed task
$ ghost status 9fe034eb
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
# Gracefully stop a task (SIGTERM) using short ID
$ ghost stop e56ed5f8
Process e56ed5f8 (8969) has been exited

# Force kill a task (SIGKILL) using short ID
$ ghost stop c3d4e5f6 --force
Process c3d4e5f6 (12347) has been killed

# Try to stop an already stopped task
$ ghost stop 9fe034eb
Error: Task operation failed: 9fe034eb-2ce7-4809-af10-2c99af15583d - Task is not running (status: exited)
```

#### Clean up old tasks

**Important**: By default, `cleanup` only removes tasks that are **older than 30 days**. This is to prevent accidental deletion of recent tasks.

```bash
# Clean up tasks older than 30 days (default)
$ ghost cleanup
Successfully deleted 5 task(s).
Deleted tasks older than 30 days with status: exited, killed.

# Clean up tasks older than 7 days
$ ghost cleanup --days 7

# Clean up ALL tasks (including recent ones) - be careful!
$ ghost cleanup --days 0

# Preview what would be deleted (dry run)
$ ghost cleanup --dry-run
The following 3 task(s) would be deleted:
Task ID                              PID      Status     Started              Command                        Directory
--------------------------------------------------------------------------------------------------------------------------------------
89afd966-5678-90ab-cdef-1234567890ab 5298     exited     2025-06-01 15:29     ./scripts/backup.sh            /home/user
6a279d57-1234-5678-abcd-ef0123456789 5203     exited     2025-06-01 15:29     ./scripts/cleanup.sh           /home/user

Note: Only tasks older than 30 days would be deleted.

# Clean up all finished tasks regardless of age
$ ghost cleanup --all
Successfully deleted 12 task(s).
Deleted all finished tasks regardless of age.

# Clean up only killed tasks older than 30 days
$ ghost cleanup --status killed
No tasks found matching cleanup criteria.
# Note: If no tasks are found, they might be too recent. Use --days or --all

# Clean up recent killed tasks (from today)
$ ghost cleanup --status killed --days 0

# Clean up all killed tasks regardless of age
$ ghost cleanup --status killed --all
```

**Cleanup Options:**
- `--days N`: Delete tasks older than N days (default: 30)
- `--all`: Delete all finished tasks regardless of age (overrides --days)
- `--status STATUS`: Filter by status (exited, killed, unknown, all)
- `--dry-run` or `-n`: Preview what would be deleted without actually deleting

### TUI Mode

Ghost includes an interactive Terminal User Interface for managing processes:

```bash
# Start TUI mode (no arguments)
$ ghost

# Or you can still use subcommands for CLI operations
$ ghost list
```

**TUI Features:**
- Real-time process status updates (refreshes every second)
- Interactive task management with search functionality
- Log viewer with line numbers and scrolling
- Keyboard navigation and filtering
- Short ID display for better readability

#### Task List Navigation

**Basic Navigation:**
- `j`/`k`: Move selection up/down
- `g`/`G`: Jump to top/bottom of list
- `Enter`: View logs for selected task
- `d`: View process details for selected task
- `s`: Send SIGTERM to selected task
- `Ctrl+K`: Send SIGKILL to selected task  
- `q`: Quit application

#### Task Filtering

**Status Filters (Tab key):**
- `Tab`: Cycle through status filters:
  - **All** → **Running** → **Exited** → **Killed** → **All**
- Filter status is shown in title bar: `[Filter: Running]`
- Tab filtering works in any mode (normal list or search filtered)

#### Search Functionality

**Search Process Names:**
- `/`: Start search mode for process names
- Type to filter tasks in real-time
- `Ctrl+n`/`Ctrl+p` or `Ctrl+j`/`Ctrl+k`: Navigate filtered results
- `Enter`: View logs for selected task
- `Tab`: Confirm search and return to task list (keeps filter active)
- `Esc`/`q`: Cancel search and return to full list

**Search in Logs:**
- `Ctrl+G`: Start search mode for log content (coming soon)

**Search Mode Features:**
- Real-time filtering as you type
- Shows match count in search bar
- Search state persists when viewing logs and returning
- Combine search filters with status filters using Tab

#### Search Workflow Examples

```
1. Basic Search:
   / → type "npm" → Ctrl+n/p to navigate → Enter to view logs

2. Search + Status Filter:
   / → type "npm" → Tab to confirm → Tab again to filter by Running
   (Footer shows: "Filtered by: 'npm' - Tab:Filter" indicating Tab is available)

3. Search + Log View:
   / → type "build" → Ctrl+n to select → Enter → view logs → Esc → still filtered

4. Multiple Filters Combined:
   / → type "npm" → Tab to confirm → Tab multiple times to cycle status filters
   (Search filter + Status filter work together)

5. Clear Search:
   In filtered state → q → back to full list
```

#### Process Details Viewer

**Process Details Features:**
- Shows comprehensive task information including PID, PGID, runtime, and working directory
- Displays environment variables with scrollable view
- Command copying functionality for easy reuse

**Process Details Keybindings:**
- `q`/`Esc`: Return to task list
- `c`: Copy command to clipboard (macOS)
- `j`/`k`: Scroll up/down in environment variables
- `Ctrl+D`/`Ctrl+U`: Page down/up in environment variables

**Information Displayed:**
- **Basic Info**: Task ID, parsed command, status with runtime, PID/PGID, working directory
- **Environment Variables**: All environment variables from task execution context
- **Interactive Features**: Scroll through long environment variable lists

#### Log Viewer

**Log Viewer Keybindings:**
- `j`/`k`: Scroll up/down
- `h`/`l`: Scroll left/right (for long lines)
- `g`/`G`: Jump to top/bottom
- `Ctrl+D`/`Ctrl+U`: Page down/up
- `/`: Search within current log (coming soon)
- `Esc`/`q`: Return to task list
- Maintains search filter state when returning

#### Status Indicators

**Task Status Colors:**
- **Green**: Running tasks
- **Blue**: Exited tasks  
- **Red**: Killed tasks
- **Gray**: Unknown status

#### Short ID Support

**ID Display:**
- Shows short IDs (e.g., `550ef353`) instead of full UUIDs for better readability
- Short IDs are unique prefixes extracted from the full UUID (before first hyphen)
- Compatible with all CLI commands (`log`, `status`, `stop`)

**Examples:**
```bash
# Full UUID: 550ef353-2065-4362-a489-f98554051064
# Short ID:  550ef353

# All these commands are equivalent:
$ ghost log 550ef353-2065-4362-a489-f98554051064  # Full UUID
$ ghost log 550ef353                              # Short ID
$ ghost status 550ef353                           # Works with any command
$ ghost stop 550ef353                             # Convenient and readable
```

**Error Handling:**
- If multiple tasks match a short ID prefix, an error is shown with suggestions
- If no tasks match, a clear "Task not found" error is displayed

### Key Features

- **No Daemon Required**: Each command is self-contained
- **Process Isolation**: Tasks run as independent processes
- **Log Persistence**: All output is captured and stored
- **Status Monitoring**: Real-time status updates via process checking
- **Cross-Platform**: Works on Unix-like systems (Linux, macOS)

### Configuration

#### Environment Variables

- `GHOST_DATA_DIR`: Override the default data directory location. This is useful for testing or running multiple instances.

#### Default Locations

**Linux:**
- Data: `$XDG_DATA_HOME`/ghost or `$HOME`/.local/share/ghost
- Logs: `$XDG_DATA_HOME`/ghost/logs or `$HOME`/.local/share/ghost/logs

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
