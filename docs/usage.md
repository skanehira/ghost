# Ghost Usage Guide

Ghost is a command-line tool for managing background processes without a persistent daemon. This guide covers everyday workflows, the interactive TUI, MCP integration, and configuration options.

## CLI Basics

### Run a command in the background

```bash
# Run a simple command
ghost run echo "Hello, World"

# Run a long-lived command
ghost run sleep 30

# Run a script
ghost run ./my_script.sh

# Run from a specific working directory
ghost run --cwd /path/to/project make build

# Inject environment variables
ghost run --env NODE_ENV=production --env PORT=3000 npm start
```

Successful executions display the generated task ID, PID, and log file path.

### List managed tasks

```bash
# Show every task
ghost list

# Filter by status
ghost list --status running
```

The list output includes task IDs, PIDs, lifecycle status, timestamps, original command, and working directory.

### Inspect task logs

```bash
# Print a task's log
ghost log 9fe034eb-2ce7-4809-af10-2c99af15583d

# Follow output in real time (Ctrl+C to stop)
ghost log -f e56ed5f8-44c8-4905-97aa-651164afd37e
```

### Check task status

```bash
# Show details for a running task
ghost status e56ed5f8-44c8-4905-97aa-651164afd37e

# Show details for a completed task
ghost status 9fe034eb-2ce7-4809-af10-2c99af15583d
```

`ghost status` reports lifecycle state, timestamps, exit code (when finished), and log location.

### Stop a running task

```bash
# Gracefully stop (SIGTERM)
ghost stop e56ed5f8-44c8-4905-97aa-651164afd37e

# Force kill (SIGKILL)
ghost stop c3d4e5f6-g7h8-9012-cdef-345678901234 --force
```

When a task is no longer running, `ghost stop` returns an error indicating the recorded status.

### Clean up finished tasks

By default, `ghost cleanup` removes tasks older than 30 days to avoid accidental deletion of recent history.

```bash
# Delete tasks older than 30 days
ghost cleanup

# Delete tasks older than 7 days
ghost cleanup --days 7

# Delete all finished tasks (be careful!)
ghost cleanup --all

# Preview deletions without removing anything
ghost cleanup --dry-run
```

Useful options:

- `--days N`: Delete tasks older than *N* days (default: 30)
- `--all`: Delete every finished task regardless of age
- `--status <STATUS>`: Restrict to `exited`, `killed`, `unknown`, or `all`
- `--dry-run`, `-n`: Print what would be deleted without performing it

## TUI Mode

Start the interactive interface by running `ghost` with no subcommand:

```bash
ghost
```

You can also open the TUI after subcommands (`ghost list`, etc.).

**TUI highlights**

- Real-time task refresh (every second)
- Interactive task management (view details, rerun, stop)
- Listening port detection when `lsof` is available
- Integrated log viewer with line numbers

**Task list keybindings**

- `j` / `k`: Move selection
- `g` / `G`: Jump to top/bottom
- `Enter`: View selected task details (ports, environment)
- `l`: Open logs for the selected task
- `r`: Rerun the selected command
- `s`: Send SIGTERM
- `Ctrl+K`: Send SIGKILL
- `Tab`: Switch between filters (All / Running / Exited / Killed)
- `q`: Quit the TUI

**Process details view**

- `j` / `k`: Scroll
- `Ctrl+D` / `Ctrl+U`: Page down/up
- `Esc`: Return to task list
- `q`: Quit the TUI

**Log viewer**

- `j` / `k`: Scroll vertically
- `h` / `l`: Scroll horizontally
- `g` / `G`: Jump to top/bottom
- `Esc`: Return to task list

## MCP Server Mode

Ghost can act as a Model Context Protocol (MCP) server for AI assistants such as Claude.

### Starting the server

```bash
ghost mcp
```

The command listens on stdio, enabling clients to send tool requests.

### Available MCP tools

- `ghost_run`: Run a command as a background process (parameters: `command`, `args`, `cwd`, `env`)
- `ghost_list`: List all managed processes (parameters: `status`, `running`)
- `ghost_stop`: Stop a running process (parameters: `id`)
- `ghost_log`: Fetch a task's log (parameters: `id`)

### Claude Desktop configuration example

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

After adding the server, you can issue natural-language commands such as:

- "Use ghost to run a web server on port 8080"
- "List all running processes managed by ghost"
- "Stop the process with ID abc123"
- "Show me the logs for process xyz789"

## Configuration

### Environment variables

- `GHOST_DATA_DIR`: Override the default data directory. Helpful for testing or running multiple instances side by side.

### Default locations

**Linux**

- Data: `$XDG_DATA_HOME/ghost` or `$HOME/.local/share/ghost`
- Logs: `$XDG_DATA_HOME/ghost/logs` or `$HOME/.local/share/ghost/logs`

**macOS**

- Data: `~/Library/Application Support/ghost/`
- Logs: `~/Library/Application Support/ghost/logs/`

For architecture and internal design notes, see `docs/architecture.md`.
