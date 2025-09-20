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
- **Process Management**: Start, stop, and monitor processes with SIGTERM/SIGKILL
- **Process Details View**: Comprehensive task information with environment variables
- **Log Management**: Automatic log file creation and real-time following
- **Search Functionality**: Real-time process name filtering with persistent state
- **Cross-View Navigation**: Seamless switching between process details and logs
- **Command Copying**: Copy task commands to clipboard (macOS)
- **Short ID Support**: Use abbreviated task IDs for convenience
- **Working Directory Tracking**: See where each command was executed with path shortening
- **Port Detection**: Automatic web server port detection for running processes
- **MCP Server Support**: Integrate with AI assistants via Model Context Protocol (`ghost mcp`)
- **Browser Integration**: One-key browser opening for web servers (o key)
- **Enhanced TUI Layout**: Optimized column ordering and directory display
- **No Daemon Required**: Simple one-shot execution model

This tool was inspired by:
- [pueue](https://github.com/Nukesor/pueue)
- [task-spooler](https://github.com/justanhduc/task-spooler)

## Installation

### Requirements

- Unix-based system (Linux, macOS, BSD)
- Rust 1.80+ (2024 edition)
- `lsof` command (optional, required for listening port detection)

### Build from source

```bash
git clone https://github.com/skanehira/ghost.git
cd ghost
cargo build --release
```

The binary will be available at `target/release/ghost`.

### Install

```bash
# Using cargo install (recommended for development)
cargo install --path .

# Or copy to local bin directory  
cp target/release/ghost ~/.local/bin/

# Or to system bin (requires sudo)
sudo cp target/release/ghost /usr/local/bin/
```

### Development Setup

For efficient development, install `just` task runner:

```bash
# Install just
cargo install just

# Build and install ghost
just install

# Other development commands
just dev                     # Install and run development mode
just test-all               # Install and run tests
just watch-install          # Auto-install on file changes
just list                   # Show available commands
```

The `just install` command runs `cargo install --path .` which builds in release mode and installs to `~/.cargo/bin/` automatically.

### Optional: Install lsof for port detection

Listening-port detection relies on the `lsof` command. Install it if your system does not already provide it:

```bash
# macOS (usually pre-installed)
brew install lsof           # If missing

# Linux
sudo apt-get install lsof   # Debian/Ubuntu
sudo yum install lsof       # RHEL/CentOS
sudo pacman -S lsof         # Arch Linux
```

Ghost works without `lsof`, but the TUI will prompt you to install it if port detection is unavailable.

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

# View partial logs (default: head 100 + tail 100)
$ ghost log 6a68d91f
Line 1
Line 2
...
Line 100

... 300 lines omitted ...

Line 401
Line 402
...
Line 500

# View all logs
$ ghost log 6a68d91f --all
Line 1
Line 2
...
Line 500

# View custom head/tail
$ ghost log 6a68d91f --head 10 --tail 5
Line 1
...
Line 10

... 485 lines omitted ...

Line 496
...
Line 500
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

#### Restart a task

```bash
# Restart a running task (stop and start again)
$ ghost restart e56ed5f8
Stopping task e56ed5f8...
Starting task e56ed5f8...
Started background process:
  Task ID: bf294251-eb9c-42e7-8c55-abeaccb81dd0
  PID: 62187
  Log file: /Users/user/Library/Application Support/ghost/logs/bf294251-eb9c-42e7-8c55-abeaccb81dd0.log
Task e56ed5f8 has been restarted successfully

# Restart a stopped task (simply run again)
$ ghost restart 9fe034eb
Starting task 9fe034eb...
Started background process:
  Task ID: d3e4f5g6-h7i8-9012-jklm-345678901234
  PID: 62188
  Log file: /Users/user/Library/Application Support/ghost/logs/d3e4f5g6-h7i8-9012-jklm-345678901234.log
Task 9fe034eb has been rerun successfully

# Force restart with SIGKILL
$ ghost restart c3d4e5f6 --force
Stopping task c3d4e5f6...
Starting task c3d4e5f6...
Task c3d4e5f6 has been restarted successfully
```

**Restart Features:**
- Preserves original working directory
- Preserves original environment variables
- Running tasks are stopped first, then restarted
- Stopped tasks are simply run again (rerun)

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
- `o`: Open browser for running web servers (automatic port detection)
- `s`: Stop task (SIGTERM - graceful termination)
- `Ctrl+K`: Kill task (SIGKILL - forced termination)  
- `q`/`Esc`: Quit application or clear search filter

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
- `Esc`/`q`: Cancel search and return to full list (also works in filtered state)

**Search in Logs:**
- Log content search feature coming soon

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
   In filtered state → q or Esc → back to full list
```

#### Process Details Viewer

**Process Details Features:**
- Shows comprehensive task information including PID, PGID, runtime, and working directory
- Displays environment variables with scrollable view
- Command copying functionality for easy reuse

**Process Details Keybindings:**
- `q`/`Esc`: Return to task list
- `l`: Switch to log view for the same task
- `c`: Copy command to clipboard (macOS)
- `j`/`k`: Scroll up/down in environment variables
- `Ctrl+D`/`Ctrl+U`: Page down/up in environment variables

**Information Displayed:**
- **Basic Info**: Task ID, parsed command, status with runtime, PID/PGID, working directory
- **Environment Variables**: All environment variables from task execution context
- **Interactive Features**: Scroll through long environment variable lists

#### Cross-Navigation Between Views

**Seamless View Switching:**
- Switch between Process Details and Log View for the same task without returning to task list
- Maintains task context when transitioning between views
- Enhances workflow efficiency for debugging and monitoring

**Navigation Flow:**
```
Task List → [Enter] → Log View → [d] → Process Details → [l] → Log View
    ↑              ↗                 ↙              ↖
    [q/Esc] ──────┘                 └────── [q/Esc]
```

#### Log Viewer

**Log Viewer Features:**
- **Auto-Scroll Mode**: Press `f` to enable tail -f like auto-scrolling
- Shows `[Auto-Scroll]` indicator when active
- Automatically scrolls to bottom when new log content arrives
- Manual scrolling (j/k/g/Ctrl+D/Ctrl+U) disables auto-scroll
- Press `f` again to toggle auto-scroll off/on

**Log Viewer Keybindings:**
- `j`/`k`: Scroll up/down (disables auto-scroll if active)
- `h`/`l`: Scroll left/right (for long lines)
- `g`/`G`: Jump to top/bottom (g disables auto-scroll)
- `f`: Toggle auto-scroll mode (like tail -f)
- `d`: Switch to process details for the same task
- `Ctrl+D`/`Ctrl+U`: Page down/up (disables auto-scroll if active)
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

#### Port Detection and Browser Integration

**Automatic Port Detection:**
- Detects listening TCP ports for running processes using `lsof`
- Shows port information in the Port column (e.g., `:3000`, `:8080`)
- Works cross-platform (Linux, macOS)
- Only displays ports for currently running processes
- Safe for non-web processes (no errors for processes without listening ports)

**Browser Integration:**
- Press `o` key to open detected web servers in browser
- Automatically constructs URL: `http://localhost:{detected_port}`
- Only works for running processes with detected ports
- Uses system default browser (macOS `open` command)

**Examples:**
```bash
# Start web servers
$ ghost run python3 -m http.server 3000
$ ghost run npm run dev   # Usually runs on port 3001
$ ghost run rails server  # Usually runs on port 3000

# In TUI mode:
# - Port column shows: :3000, :3001, etc.
# - Press 'o' on any running web server to open in browser
# - Non-web processes show '-' in Port column
```

**Enhanced Directory Display:**
- Directory paths are shortened for better readability
- Home directory shown as `~` 
- Middle path components abbreviated to first character
- Example: `/Users/john/Projects/my-app/src` → `~/P/m/src`
- Optimized column layout with Started time moved to first column

### MCP Server Mode

Ghost supports the Model Context Protocol (MCP), allowing it to be used as an MCP server for AI assistants like Claude.

#### Starting the MCP Server

```bash
ghost mcp
```

This starts an MCP server on stdio that can be used by AI assistants.

#### Available MCP Tools

- **ghost_run**: Run a command as a background process
  - Parameters: command, args, cwd, env
- **ghost_list**: List all managed processes
  - Parameters: status (filter), running (boolean)
- **ghost_stop**: Stop a running process
  - Parameters: id
- **ghost_log**: Get logs for a process
  - Parameters: id

#### Configuration for Claude Desktop

Add to your Claude Desktop configuration:

```json
{
  "mcpServers": {
    "ghost": {
      "command": "/path/to/ghost",
      "args": ["mcp"]
    }
  }
}
```

Once configured, you can use natural language to interact with ghost through Claude:
- "Use ghost to run a web server on port 8080"
- "List all running processes managed by ghost"
- "Stop the process with ID abc123"
- "Show me the logs for process xyz789"

### Key Features

- **No Daemon Required**: Each command is self-contained
- **Process Isolation**: Tasks run as independent processes
- **Log Persistence**: All output is captured and stored
- **Status Monitoring**: Real-time status updates via process checking
- **MCP Integration**: Control ghost through AI assistants
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
