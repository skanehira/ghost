![GitHub Repo stars](https://img.shields.io/github/stars/skanehira/ghost?style=social)
![GitHub](https://img.shields.io/github/license/skanehira/ghost)
![GitHub all releases](https://img.shields.io/github/downloads/skanehira/ghost/total)
![GitHub CI Status](https://img.shields.io/github/actions/workflow/status/skanehira/ghost/ci.yaml?branch=main)
![GitHub Release Status](https://img.shields.io/github/v/release/skanehira/ghost)

# Ghost
This is a simple shell command management tool.

It has the following features:

- Manage shell commands with a TUI
- Background execution of shell commands
- No daemon required

This tool was inspired by the following:

- [pueue](https://github.com/Nukesor/pueue)
- [task-spooler](https://github.com/justanhduc/task-spooler)

## Installation

### Build from source

```bash
git clone https://github.com/skanehira/ghost.git
cd ghost
cargo build --release
```

The binary will be available at `target/release/ghost`.

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
Task ID  PID      Status     Started              Command                       
--------------------------------------------------------------------------------
e56ed5f8 8969     exited     2025-07-01 15:36     sleep 30                      
9fe034eb 8730     exited     2025-07-01 15:35     echo Hello, World             
cb5b7275 8349     running    2025-07-01 15:34     echo Hello, World             
98765f79 5404     killed     2025-07-01 15:29     ./scripts/hello_loop.sh       
89afd966 5298     exited     2025-07-01 15:29     ./scripts/hello_loop.sh       

# Filter by status
$ ghost list --status exited
Task ID  PID      Status     Started              Command                       
--------------------------------------------------------------------------------
e56ed5f8 8969     exited     2025-07-01 15:36     sleep 30                      
9fe034eb 8730     exited     2025-07-01 15:35     echo Hello, World             
89afd966 5298     exited     2025-07-01 15:29     ./scripts/hello_loop.sh       
6a279d57 5203     exited     2025-07-01 15:29     ./scripts/hello_loop.sh       
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
Started: 2025-07-01 15:36:23
Log file: /Users/user/Library/Application Support/ghost/logs/e56ed5f8-44c8-4905-97aa-651164afd37e.log

# Get detailed information about a completed task
$ ghost status 9fe034eb-2ce7-4809-af10-2c99af15583d
Task: 9fe034eb-2ce7-4809-af10-2c99af15583d
PID: 8730
Status: exited
Command: echo Hello, World
Started: 2025-07-01 15:35:36
Finished: 2025-07-01 15:36:10
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

#### Kill a process by PID

```bash
# Kill a process directly by PID
$ ghost kill 8969
Process 8969 killed successfully.

# Try to kill a non-existent PID
$ ghost kill 9999
Error: Unix system error: ESRCH: No such process

# Try to kill with invalid PID format
$ ghost kill abc123
Error: invalid digit found in string
```

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

# Monitor multiple tasks
$ ghost list
Task ID  PID      Status     Started              Command                       
--------------------------------------------------------------------------------
d4e5f6g7 12348    running    2024-01-15 10:35     python -m http.server 8080    
e5f6g7h8 12349    exited     2024-01-15 10:36     npm test                      
f6g7h8i9 12350    running    2024-01-15 10:37     rsync -av /home/user/ /backup/

$ ghost status d4e5f6g7  # Use the task ID from list output
Task: d4e5f6g7-h8i9-0123-defg-456789012345
PID: 12348
Status: running
Command: python -m http.server 8080
Started: 2024-01-15 10:35:42
Log file: /Users/user/Library/Application Support/ghost/logs/d4e5f6g7-h8i9-0123-defg-456789012345.log

# Check logs of a running task
$ ghost log d4e5f6g7
Serving HTTP on 0.0.0.0 port 8080 (http://0.0.0.0:8080/) ...
192.168.1.100 - - [15/Jan/2024 10:36:15] "GET / HTTP/1.1" 200 -
192.168.1.100 - - [15/Jan/2024 10:36:20] "GET /favicon.ico HTTP/1.1" 404 -

# Stop a task when done
$ ghost stop d4e5f6g7
Process d4e5f6g7-h8i9-0123-defg-456789012345 (12348) has been exited
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

### Architecture

Ghost uses a one-shot design where:
1. **SQLite Database**: Stores task metadata and status
2. **File-based Logs**: Each task gets its own log file
3. **Process Groups**: Proper cleanup and signal handling
4. **Lazy Evaluation**: Status updates only when needed

This approach eliminates the complexity of daemon management while providing reliable background process execution.
