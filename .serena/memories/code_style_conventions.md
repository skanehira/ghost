# Ghost Code Style and Conventions

## Rust Formatting Rules
- **ALWAYS use inline format strings with embedded expressions**
  - Use `format!("text {variable}")` instead of `format!("text {}", variable)`
  - Use `println!("value: {x}")` instead of `println!("value: {}", x)`
  - Use `writeln!(file, "Log line {i}")` instead of `writeln!(file, "Log line {}", i)`
  - Applies to all formatting macros: `format!`, `print!`, `println!`, `write!`, `writeln!`, `eprintln!`

## Project Structure
```
src/
├── main.rs              # Entry point with CLI parsing
├── lib.rs               # Library crate
└── app/                 # Application modules
    ├── commands.rs      # Command implementations
    ├── config.rs        # Configuration
    ├── display.rs       # Display formatting
    ├── error.rs         # Error types
    ├── process.rs       # Process management
    ├── storage/         # Database and task management
    │   ├── database.rs
    │   ├── task.rs
    │   └── task_repository.rs
    ├── tui/             # Terminal UI components
    │   ├── app.rs
    │   ├── task_list.rs
    │   └── log_viewer_scrollview.rs
    └── helpers/         # Utility functions
```

## Error Handling
- Use `thiserror` for error definitions
- Return `Result<T, Error>` for fallible operations
- Provide meaningful error messages

## Testing
- Unit tests in the same file as the code
- Integration tests in `tests/` directory
- Use `pretty_assertions` for better test output
- Use `tempfile` for test file operations

## Dependencies
- Keep dependencies minimal
- Use well-maintained crates
- Pin versions in Cargo.toml
- Regular security audits via GitHub Actions

## CI/CD Conventions
- All code must pass formatting check
- All clippy warnings must be resolved
- Tests must pass on Linux, macOS, and Windows
- Benchmarks run on every push
- Releases are automated via tags (v*)