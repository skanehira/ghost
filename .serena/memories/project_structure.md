# Ghost Project Structure

## Core Directories
```
ghost/
├── src/                    # Source code
│   ├── main.rs            # Entry point, CLI parsing
│   ├── lib.rs             # Library crate (mainly tests)
│   └── app/               # Main application logic
│       ├── commands.rs    # Command implementations (run, list, log, etc.)
│       ├── tui/           # Terminal UI components
│       │   ├── app.rs     # Main TUI application state
│       │   ├── task_list.rs # Task list view
│       │   ├── log_viewer_scrollview.rs # Log viewer with scrolling
│       │   └── process_details.rs # Process details view
│       ├── storage/       # Database and persistence
│       │   ├── database.rs # SQLite connection management
│       │   ├── task.rs    # Task model
│       │   └── task_repository.rs # Task CRUD operations
│       └── helpers/       # Utility functions
│           ├── port_utils.rs # Port detection
│           ├── process_utils.rs # Process management
│           └── time.rs    # Time formatting
├── tests/                 # Integration tests
├── benches/              # Benchmarks
├── examples/             # Example scripts
└── .github/workflows/    # CI/CD configurations
```

## Key Components
- **CLI**: Clap-based command parsing in main.rs
- **TUI**: Ratatui-based interactive interface
- **Storage**: SQLite database for task metadata
- **Process Management**: Unix signal handling with nix crate