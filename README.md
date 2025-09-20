![GitHub Repo stars](https://img.shields.io/github/stars/skanehira/ghost?style=social)
![GitHub](https://img.shields.io/github/license/skanehira/ghost)
![GitHub all releases](https://img.shields.io/github/downloads/skanehira/ghost/total)
![GitHub CI Status](https://img.shields.io/github/actions/workflow/status/skanehira/ghost/ci.yaml?branch=main)
![GitHub Release Status](https://img.shields.io/github/v/release/skanehira/ghost)

# Ghost

Ghost is a simple background process manager for Unix systems (Linux, macOS, BSD).

<div align="center">
  <img src="./images/logo.png" width=300 />
</div>

## Features

![](./images/ghost.png)

- **Background Process Execution**: Run commands in the background without a daemon
- **TUI Mode**: Interactive terminal UI for managing processes
- **Process Management**: Start, stop, and monitor processes
- **Port Detection**: Real-time listening port detection for running processes
- **Log Management**: Automatic log file creation and real-time following
- **Working Directory Tracking**: See where each command was executed
- **MCP Server Support**: Integrate with AI assistants via Model Context Protocol
- **No Daemon Required**: Simple one-shot execution model

This tool was inspired by:
- [pueue](https://github.com/Nukesor/pueue)
- [task-spooler](https://github.com/justanhduc/task-spooler)

## Installation

### Requirements

- Unix-based system (Linux, macOS, BSD)
- Rust 1.80+ (2024 edition)
- `lsof` command (for port detection feature, optional)

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

### Optional: Install lsof for port detection

For the port detection feature to work, you need `lsof` installed:

```bash
# macOS (usually pre-installed)
brew install lsof           # If not available

# Linux
sudo apt-get install lsof   # Debian/Ubuntu
sudo yum install lsof       # RHEL/CentOS
sudo pacman -S lsof         # Arch Linux
```

Note: Ghost works perfectly without `lsof`, but port detection will show installation instructions in the TUI.

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
$ ghost log -f e56ed5f8-44c8-4905-97aa-651164afd37e
Following logs for task e56ed5f8-44c8-4905-97aa-651164afd37e (Ctrl+C to stop):
Log file: /Users/user/Library/Application Support/ghost/logs/e56ed5f8-44c8-4905-97aa-651164afd37e.log
----------------------------------------
Processing file 1/10
Processing file 2/10
...
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
- Interactive task management
- Process details view with listening ports
- Log viewer with line numbers
- Keyboard navigation

**Task List Keybindings:**
- `j`/`k`: Move selection up/down
- `g`/`G`: Jump to top/bottom of list
- `Enter`: View process details (including listening ports)
- `l`: View logs for selected task
- `r`: Rerun selected command
- `s`: Send SIGTERM to selected task
- `Ctrl+K`: Send SIGKILL to selected task  
- `q`: Quit
- `Tab`: Switch between task filters (All/Running/Exited/Killed)

**Process Details Keybindings:**
- `j`/`k`: Scroll up/down through environment variables
- `Ctrl+D`/`Ctrl+U`: Page down/up
- `q`: Quit application
- `Esc`: Return to task list

**Log Viewer Keybindings:**
- `j`/`k`: Scroll up/down
- `h`/`l`: Scroll left/right (for long lines)
- `g`/`G`: Jump to top/bottom
- `Esc`: Return to task list

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
- **Port Detection**: Real-time listening port detection via `lsof`
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
