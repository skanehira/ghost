# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ghost is a lightweight background process manager for Unix systems that provides daemon-free process management with a TUI interface and MCP server capabilities. Built in Rust edition 2024 with minimum toolchain 1.80.

## Development Commands

### Build
```bash
cargo build                    # Debug build
cargo build --release         # Release build
```

### Test
```bash
cargo test                    # Run all unit/integration tests
cargo nextest run            # Faster parallel test execution with better output
./e2e/run_all_tests.sh       # Run end-to-end shell script tests
```

### Format & Lint
```bash
cargo fmt                     # Format code
cargo fmt --all -- --check    # Check formatting without changes
cargo clippy                  # Run linter
cargo clippy -- -D warnings   # Treat warnings as errors
```

### Coverage
```bash
cargo llvm-cov nextest --lcov --output-path lcov.info  # Generate coverage report
```

## Architecture

### Core Modules
- `src/main.rs`: CLI entry point using clap v4.5.47 with subcommands
- `src/mcp/`: MCP server implementation using rust-mcp-sdk
- `src/app/`:
  - `commands.rs`: Command implementations (run, list, log, stop, status, cleanup)
  - `process.rs`: Process spawning with `setsid()` for process group isolation
  - `port_detector.rs`: Real-time port detection via `lsof` command parsing
  - `storage/`: SQLite operations with WAL mode, task CRUD, cleanup logic
  - `tui/`: Terminal UI with ratatui v0.29, EventStream for non-blocking input
  - `helpers/`: File watching, time formatting, task validation utilities

### Database Schema
SQLite database at `$GHOST_DATA_DIR/tasks.db`:
- Tasks table with UUID primary key, process info (pid, pgid), timestamps
- Indexes on status, pid, started_at for query performance
- WAL mode enabled for concurrent access

### Process Management
- Spawns processes in new process groups using `libc::setpgid`
- Redirects stdout/stderr to log files in `$GHOST_DATA_DIR/logs/`
- Signal handling: SIGTERM (graceful), SIGKILL (forced), signal 0 (existence check)
- Status tracking: running, exited, killed, unknown

### TUI Components
- Task list with filtering (All/Running/Exited/Killed)
- Real-time status updates (1-second refresh via tokio timer)
- Log viewer with 10k line memory limit using tui-scrollview
- Process details view showing command, environment, ports

## Dependencies

Key dependencies with specific versions:
- **clap** 4.5.47: Command-line parsing with derive macros
- **ratatui** 0.29: Terminal UI framework
- **rusqlite** 0.37: SQLite with bundled engine
- **tokio** 1.47: Async runtime with full features
- **crossterm** 0.29: Cross-platform terminal handling
- **nix** 0.30: Unix process/signal operations

## Testing Approach

### Unit/Integration Tests
- Database operations with tempfile for isolation
- Process lifecycle management with mock processes
- TUI component state transitions
- Port detection parsing edge cases

### E2E Tests
Shell script tests in `e2e/` directory:
- `001_run_command.sh`: Basic process spawning
- `002_list_command.sh`: Task listing and filtering
- `003_log_command.sh`: Log viewing and following
- `004_stop_status_commands.sh`: Process termination
- `005_cwd_functionality.sh`: Working directory handling
- `006_cleanup_command.sh`: Database maintenance

Helper script `e2e/test_helpers.sh` provides utility functions.

## CI/CD

GitHub Actions workflows:
- **ci.yaml**: Format, clippy, build, test on Linux/macOS
- **audit.yaml**: Security vulnerability scanning
- **release.yaml**: Automated binary releases
- **dependabot-auto-merge.yaml**: Dependency updates

Uses cargo-nextest for parallel test execution and cargo-llvm-cov for coverage on Linux.

## Environment Configuration

- `GHOST_DATA_DIR`: Override default data directory
- Default paths:
  - Linux: `~/.local/share/ghost/`
  - macOS: `~/Library/Application Support/ghost/`

## Rust Formatting Rules

- Use inline format strings: `format!("text {variable}")` not `format!("text {}", variable)`
- Applies to all formatting macros: `println!`, `write!`, `eprintln!`, etc.

## MCP Server Mode

Ghost includes an MCP (Model Context Protocol) server for AI assistant integration:
```bash
ghost mcp  # Run as MCP server over stdio
```

Implements tools: ghost_run, ghost_list, ghost_stop, ghost_log, ghost_status