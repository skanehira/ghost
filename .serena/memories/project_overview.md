# Ghost Project Overview

Ghost is a simple background process manager for Unix systems (Linux, macOS, BSD) written in Rust.

## Purpose
- Run commands in the background without requiring a daemon
- Provide TUI (Terminal User Interface) for interactive process management
- Manage process lifecycle (start, stop, monitor)
- Automatic log file creation and management
- Track working directory for each command

## Key Features
- No daemon required - simple one-shot execution model
- Process isolation - tasks run as independent processes
- Persistent logs for all output
- Real-time status monitoring
- Cross-platform support for Unix-like systems

## Technical Stack
- **Language**: Rust (2024 edition)
- **Rust toolchain**: 1.88
- **Database**: SQLite (with bundled driver)
- **TUI Framework**: Ratatui with tui-scrollview
- **Async Runtime**: Tokio
- **CLI Framework**: Clap v4.5
- **Process Management**: Unix signals via nix crate

## Architecture
- SQLite database stores task metadata (`ghost.db`)
- File-based logs (each task gets its own log file)
- Process groups using `setsid()` for proper signal handling
- Status monitoring via signal 0
- Non-blocking TUI with async event streams

## Data Locations
**Linux:**
- Data: `$XDG_DATA_HOME/ghost` or `$HOME/.local/share/ghost`
- Logs: `$XDG_DATA_HOME/ghost/logs` or `$HOME/.local/share/ghost/logs`

**macOS:**
- Data: `~/Library/Application Support/ghost/`
- Logs: `~/Library/Application Support/ghost/logs/`