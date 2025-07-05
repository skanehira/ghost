# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ghost is a simple shell command management tool written in Rust that provides:
- TUI-based shell command management
- Background execution of shell commands
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

## Architecture

The project is in early development stage with minimal structure:
- `src/main.rs`: Entry point with CLI argument parsing using clap
- `src/lib.rs`: Library crate (currently empty except for tests)
- `benches/`: Benchmark tests

The application uses:
- **clap** v4.5.31 for command-line argument parsing
- Rust edition 2024
- Rust toolchain 1.87

## CI/CD

GitHub Actions workflows are configured for:
- **CI** (.github/workflows/ci.yaml): Runs on push/PR, includes format check, clippy, build, and tests across Linux/macOS/Windows
- **Audit** (.github/workflows/audit.yaml): Security vulnerability scanning
- **Benchmark** (.github/workflows/benchmark.yaml): Performance benchmarking
- **Release** (.github/workflows/release.yaml): Automated releases

## Testing Approach

- Unit tests use standard `cargo test`
- CI uses `cargo-nextest` for better test output and performance
- Code coverage is generated with `cargo-llvm-cov` on Linux CI runs

## Rust Formatting Rules

- ALWAYS use inline format strings with embedded expressions when possible
- Use `format!("text {variable}")` instead of `format!("text {}", variable)`
- Use `println!("value: {x}")` instead of `println!("value: {}", x)`
- Use `writeln!(file, "Log line {i}")` instead of `writeln!(file, "Log line {}", i)`
- This applies to all formatting macros: `format!`, `print!`, `println!`, `write!`, `writeln!`, `eprintln!`, etc.
