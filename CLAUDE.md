# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ghost is a simple shell command management tool written in Rust that provides:
- TUI-based shell command management
- Background execution of shell commands
- Real-time listening port detection for running processes
- No daemon required

## Development Commands

### Build
```bash
cargo build                    # Debug build
cargo build --release         # Release build
```

### Test
```bash
cargo test                    # Run all tests
cargo test <test_name>        # Run specific test
cargo nextest run            # Run tests with nextest (faster, better output)
```

### Format & Lint
```bash
cargo fmt                     # Format code
cargo fmt --all -- --check    # Check formatting without changes
cargo clippy                  # Run linter
cargo clippy -- -D warnings   # Fail on warnings
```

### Coverage
```bash
cargo llvm-cov nextest --lcov --output-path lcov.info  # Generate coverage report
```

### Benchmarks
```bash
cargo bench                   # Run benchmarks
```

### System Requirements

For full port detection functionality:
```bash
# macOS (usually pre-installed)
brew install lsof           # If not available

# Linux
sudo apt-get install lsof   # Debian/Ubuntu
sudo yum install lsof       # RHEL/CentOS
sudo pacman -S lsof         # Arch Linux
```

Note: Port detection gracefully degrades when `lsof` is not available, showing installation instructions in the UI.

## Architecture

### Core Components

- `src/main.rs`: Entry point with CLI argument parsing using clap
- `src/lib.rs`: Library crate with module exports
- `src/app/`: Application logic modules
  - `config.rs`: Configuration management
  - `commands.rs`: Command execution handling
  - `process.rs`: Process spawning and management
  - `port_detector.rs`: Real-time listening port detection
  - `storage/`: Database and task persistence
  - `tui/`: Terminal user interface components
- `tests/`: Integration and unit tests
- `benches/`: Performance benchmarks

### Key Features

#### Port Detection
- **Cross-platform support**: macOS and Linux via `lsof` command
- **Performance optimized**: Single lsof availability check per application run
- **Error handling**: User-friendly messages when lsof is not installed
- **Real-time display**: Shows TCP/UDP listening ports in process details view
- **Robust parsing**: Handles malformed lsof output gracefully

#### Process Management
- Background process execution without daemon
- Process status monitoring and lifecycle management
- Environment variable and working directory support

#### TUI Interface
- Process list with filtering and navigation
- Detailed process view with port information
- Log viewer with scrolling and search capabilities

### Dependencies

The application uses:
- **clap** v4.5.31 for command-line argument parsing
- **ratatui** for terminal user interface
- **rusqlite** for local database storage
- **crossterm** for cross-platform terminal handling
- Rust edition 2024
- Rust toolchain 1.88

## CI/CD

GitHub Actions workflows are configured for:
- **CI** (.github/workflows/ci.yaml): Runs on push/PR, includes format check, clippy, build, and tests across Linux/macOS/Windows
- **Audit** (.github/workflows/audit.yaml): Security vulnerability scanning
- **Benchmark** (.github/workflows/benchmark.yaml): Performance benchmarking
- **Release** (.github/workflows/release.yaml): Automated releases

## Testing Approach

- **Unit tests**: Standard `cargo test` with comprehensive edge case coverage
- **Integration tests**: Cross-platform testing using Docker for Linux compatibility
- **TUI tests**: Terminal interface testing with expected output validation
- **Port detection tests**: TCP server integration tests for real-world scenarios
- **Error handling tests**: Malformed output, permission denied, and partial data scenarios
- **CI**: Uses `cargo-nextest` for better test output and performance
- **Coverage**: Generated with `cargo-llvm-cov` on Linux CI runs

## Rust Formatting Rules

- ALWAYS use inline format strings with embedded expressions when possible
- Use `format!("text {variable}")` instead of `format!("text {}", variable)`
- Use `println!("value: {x}")` instead of `println!("value: {}", x)`
- Use `writeln!(file, "Log line {i}")` instead of `writeln!(file, "Log line {}", i)`
- This applies to all formatting macros: `format!`, `print!`, `println!`, `write!`, `writeln!`, `eprintln!`, etc.
