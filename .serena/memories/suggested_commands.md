# Ghost Development Commands

## Build Commands
```bash
cargo build                    # Debug build
cargo build --release         # Release build
```

## Testing Commands
```bash
cargo test                    # Run all tests
cargo test <test_name>        # Run specific test
cargo nextest run            # Run tests with nextest (faster, better output)
```

## Code Quality Commands
```bash
cargo fmt                     # Format code
cargo fmt --all -- --check    # Check formatting without changes
cargo clippy                  # Run linter
cargo clippy -- -D warnings   # Fail on warnings
```

## Coverage
```bash
cargo llvm-cov nextest --lcov --output-path lcov.info  # Generate coverage report
```

## Benchmarks
```bash
cargo bench                   # Run benchmarks
```

## Ghost Usage Commands
```bash
ghost run <command>           # Run a command in background
ghost list                    # List all tasks
ghost log <task-id>           # View task logs
ghost log -f <task-id>        # Follow logs in real-time
ghost status <task-id>        # Check task status
ghost stop <task-id>          # Stop a task (SIGTERM)
ghost stop --force <task-id>  # Force kill a task (SIGKILL)
ghost cleanup                 # Clean up old tasks (>30 days)
ghost cleanup --days N        # Clean up tasks older than N days
ghost cleanup --all           # Clean up all finished tasks
ghost                         # Start TUI mode
```

## Darwin System Commands
```bash
# File operations
ls -la                        # List files with details
find . -name "*.rs"          # Find files by pattern
grep -r "pattern" .          # Search in files recursively

# Process management
ps aux | grep ghost          # Find ghost processes
kill -TERM <pid>             # Send SIGTERM
kill -KILL <pid>             # Send SIGKILL

# Git operations
git status                   # Check repo status
git diff                     # Show changes
git log --oneline -10        # Recent commits
git tag -l | sort -V         # List tags sorted by version
```