window.BENCHMARK_DATA = {
  "lastUpdate": 1751811447036,
  "repoUrl": "https://github.com/skanehira/ghost",
  "entries": {
    "Rust Benchmark": [
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "de9cbfcbc2b5d711f9ba3a3166544e12d326cd54",
          "message": "chore: format",
          "timestamp": "2025-07-02T01:42:52+09:00",
          "tree_id": "155dc5c5badb81bfd98a9986076255dfe2f2f205",
          "url": "https://github.com/skanehira/ghost/commit/de9cbfcbc2b5d711f9ba3a3166544e12d326cd54"
        },
        "date": 1751389383175,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.95,
            "range": "± 0.20",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "08c95ac988aa94e8b1ce380cbdffda3fc09bbe88",
          "message": "feat: add task history cleanup functionality (#2)\n\n* refactor: reorganize module structure for better maintainability\n\nPhase 1 Refactoring:\n- Rename core/command → core/process with cleaner API\n  - spawn_background_process(), kill(), exists() functions\n  - Use as process::spawn_background_process() for clarity\n- Extract cmd_* functions from main.rs → core/commands module\n  - Reduced main.rs from 286 to 92 lines\n  - Better separation of concerns\n- Unify error handling with GhostError type\n  - Replace Box<dyn Error> with type-safe errors\n  - Centralized error definitions\n- Rename core module → app to avoid conflicts with Rust's core\n  - ghost::app::process instead of ghost::core::process\n  - More idiomatic module naming\n\nAll tests pass, clippy clean, examples updated.\n\n* refactor: extract display logic to dedicated module\n\nPhase 2.1: Create app/display module\n- Extract table formatting logic from commands.rs\n- Add print_task_list() for consistent task table display\n- Add print_task_details() for status command output\n- Add print_process_started() for run command output\n- Add print_log_follow_header() for log follow mode\n- Reduce commands.rs complexity by ~50 lines\n- Improve single responsibility principle\n\nAll tests pass: clippy ✓, build ✓, unit tests ✓, E2E tests ✓\n\n* refactor: extract process state management logic\n\nPhase 2.2: Create app/process_state module\n- Extract process status checking logic from storage.rs\n- Add update_task_status_if_needed() for centralized status updates\n- Add determine_status_after_kill() for consistent kill status logic\n- Add verify_process_status() for process validation\n- Improve separation of concerns between storage and process state\n- Add comprehensive unit tests for process state logic\n\nAll tests pass: clippy ✓, build ✓, unit tests ✓ (9 passed), E2E tests ✓\n\n* refactor(Phase 2.3): extract configuration management to app/config module\n\n- Create centralized Config struct with data_dir, log_dir, db_path\n- Move environment variable parsing from commands.rs to config::env module\n- Update storage.rs and process.rs to use config module for paths\n- Improve path management consistency across the application\n\n* refactor(Phase 3): improve code organization and maintainability\n\n- Split cmd_run function into smaller, focused helper functions\n- Simplify display format processing with utility functions\n- Extract common helper functions to app/helpers module\n- Centralize database connection, task retrieval, file reading, and validation logic\n- Improve code reusability and reduce duplication across commands\n- All tests pass (14 unit tests, 5 E2E tests)\n\n* feat: implement comprehensive task history cleanup functionality\n\n## Added cleanup command with flexible options:\n- `ghost cleanup --days N` - Delete tasks older than N days (default: 30)\n- `ghost cleanup --status STATUS` - Filter by status (exited, killed, all)\n- `ghost cleanup --dry-run` - Preview what would be deleted\n- `ghost cleanup --all` - Delete all finished tasks regardless of age\n\n## Key features:\n- Safety protection: Cannot delete running tasks\n- Flexible status filtering: exited, killed, unknown, or all\n- Dry-run mode for safe preview before deletion\n- Comprehensive error handling and user feedback\n- Maintains existing functionality (all E2E tests pass)\n\n## Implementation details:\n- Added storage functions: get_cleanup_candidates, cleanup_tasks_by_criteria\n- Enhanced commands.rs with cleanup function and status parsing\n- Updated main.rs CLI with new Cleanup subcommand\n- Proper error handling for invalid status combinations\n\n## Usage examples:\n- `ghost cleanup --dry-run` - Preview default cleanup (30+ days old)\n- `ghost cleanup --days 7` - Delete tasks older than 7 days\n- `ghost cleanup --status exited --days 0` - Delete all exited tasks\n- `ghost cleanup --all --dry-run` - Preview deleting all finished tasks\n\nTested with 237 tasks: successfully reduced to 18 killed tasks only.\nAll unit tests (14) and E2E tests (5) passing.\n\n* feat: add comprehensive E2E tests for cleanup functionality\n\n- Add test_zzz_cleanup_command.sh with full cleanup feature testing\n- Fix test_list_command.sh task ID matching (use 5 chars instead of 8)\n- Fix SQL queries in storage.rs to properly handle finished_at NULL values\n- All E2E tests now pass (6/6)\n\nFeatures tested:\n- Dry-run functionality with accurate task counts\n- Status filtering (exited, killed, running)\n- Age-based cleanup with days parameter\n- --all flag for cleaning all finished tasks\n- Protection against cleaning running tasks\n- Invalid status handling and error cases",
          "timestamp": "2025-07-02T04:51:07+09:00",
          "tree_id": "54956f10d259e105a7aa5d72024e051a0f73508d",
          "url": "https://github.com/skanehira/ghost/commit/08c95ac988aa94e8b1ce380cbdffda3fc09bbe88"
        },
        "date": 1751399546649,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.94,
            "range": "± 0.45",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "9a1c6f1b1e5a08716fda1dbef53032560c54fb9c",
          "message": "feat: implement log --follow functionality and fix cwd bug\n\n## Phase 2 Implementation (TDD approach)\n- Implement real-time log following with `ghost log --follow`\n- Replace placeholder message with functional tail -f behavior\n- Add file monitoring for incremental log updates\n\n## CWD Bug Fix (TDD approach)\n- Fix cwd parameter not being used in process execution\n- Add cwd parameter to spawn_background_process function\n- Display working directory in task status output\n- Add comprehensive E2E test for cwd functionality\n- Add unit test for cwd process spawning\n\n## Changes\n- `helpers.rs`: Add follow_log_file() for real-time log streaming\n- `commands.rs`: Use follow_log_file() instead of placeholder message\n- `process.rs`: Add cwd parameter and apply to Command::current_dir()\n- `display.rs`: Show working directory in task status\n- `examples/spawn_script.rs`: Update function signature\n\n## Testing\n- All existing E2E tests pass\n- CWD functionality fully tested with E2E coverage\n- Log follow functionality implemented (manual testing confirmed)\n\nFollows t-wada TDD cycle: Red → Green → Refactor",
          "timestamp": "2025-07-02T05:35:03+09:00",
          "tree_id": "a85494f2f1f570736c703e9448fa2d534c485296",
          "url": "https://github.com/skanehira/ghost/commit/9a1c6f1b1e5a08716fda1dbef53032560c54fb9c"
        },
        "date": 1751402294811,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.95,
            "range": "± 0.17",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c1cff347820109f00d53ed90edbd085464b31828",
          "message": "feat: add log file deletion to cleanup functionality (#3)\n\n- Implement log file deletion when tasks are cleaned up via `ghost cleanup`\n- Add E2E test to verify log files are deleted during cleanup\n- Refactor cleanup_tasks_by_criteria to use task IDs for more efficient deletion\n- Remove unnecessary follow flag test from log command E2E test\n- Fix clippy warning for uninlined format args\n\nFollowing TDD approach (Red → Green → Refactor):\n- Red: Added failing test for log file deletion\n- Green: Implemented minimal functionality to pass test\n- Refactor: Improved code efficiency and maintainability",
          "timestamp": "2025-07-02T05:56:13+09:00",
          "tree_id": "4916360f874e628e8114e1c2fa8efab8c2190aec",
          "url": "https://github.com/skanehira/ghost/commit/c1cff347820109f00d53ed90edbd085464b31828"
        },
        "date": 1751403448831,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.95,
            "range": "± 0.09",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "dfb258cab17d5980b09e300334d7501bd1981cf1",
          "message": "refactor: remove unnecessary wrapper functions from helpers.rs\n\n- Remove simple wrapper functions that added no value:\n  - init_db_connection() → use storage::init_database() directly\n  - get_task_by_id() → use storage::get_task() directly\n  - get_task_with_status_update() → use storage::update_task_status_by_process_check() directly\n  - print_file_content() → use print\\! macro directly\n\n- Keep meaningful helper functions with actual logic:\n  - read_file_content() (file existence check and error handling)\n  - follow_log_file() (complex file monitoring implementation)\n  - validate_task_running() (task state validation logic)\n\n- Update all call sites in commands.rs to use direct function calls\n- Remove unnecessary imports from helpers.rs\n\nThis improves code clarity by removing unnecessary abstraction layers.",
          "timestamp": "2025-07-02T06:10:52+09:00",
          "tree_id": "91879e4f0ac1b8f2d8d381d1b20316da71697888",
          "url": "https://github.com/skanehira/ghost/commit/dfb258cab17d5980b09e300334d7501bd1981cf1"
        },
        "date": 1751404447918,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.95,
            "range": "± 0.13",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "ad67cc91114e20143f1fa3ecd5050ea10f8ba175",
          "message": "feat: Implement async log following with Ctrl-C support\n\nRefactored the `log --follow` functionality to be fully asynchronous\nusing Tokio. This provides a more robust and responsive experience.\n\nKey changes:\n- Replaced the blocking, synchronous file reading loop with an\n  asynchronous implementation in `helpers::follow_log_file`.\n- Switched from `notify::RecommendedWatcher` to `notify::PollWatcher`\n  to ensure file changes are detected reliably across different\n  environments and file systems.\n- Introduced `tokio::select!` to concurrently handle file change\n  events and `Ctrl-C` signals, allowing the user to gracefully\n  stop the log following at any time.\n- Added the `tokio` runtime and updated the `main` function to be\n  asynchronous to support these changes.",
          "timestamp": "2025-07-02T07:15:02+09:00",
          "tree_id": "30b5265c2fb539d0249d9d5a0f7abc920dc1aa3c",
          "url": "https://github.com/skanehira/ghost/commit/ad67cc91114e20143f1fa3ecd5050ea10f8ba175"
        },
        "date": 1751408363934,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.97,
            "range": "± 3.05",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "a72d07b286639cd69917c5aba1875e5827166a35",
          "message": "refactor: rename E2E test files with numbered prefixes\n\n- Rename all E2E test files from test_*.sh to 001_*.sh format\n- Update run_all_tests.sh to use new naming pattern [0-9][0-9][0-9]_*.sh\n- Remove zzz prefix from cleanup test, now uses 007_ prefix\n- Tests now run in logical order:\n  001_run_command.sh - Basic process spawning\n  002_list_command.sh - Process listing\n  003_log_command.sh - Log display\n  004_stop_status_commands.sh - Process control\n  005_kill_command.sh - Force termination\n  006_cwd_functionality.sh - Working directory\n  007_cleanup_command.sh - Database cleanup (runs last)\n\nThis provides better control over test execution order and makes dependencies clear.",
          "timestamp": "2025-07-02T09:07:25+09:00",
          "tree_id": "980a2ab34c340b0335deddc50dd46576539eccbd",
          "url": "https://github.com/skanehira/ghost/commit/a72d07b286639cd69917c5aba1875e5827166a35"
        },
        "date": 1751423156545,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.95,
            "range": "± 0.24",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "ab3433751f75814351ee5d722ef6e441c412fe20",
          "message": "refactor: consolidate timestamp generation into helper function\n\n- Add now_timestamp() helper to eliminate code duplication\n- Replace 8 instances of SystemTime::now().duration_since()... pattern\n- Centralizes Unix timestamp generation logic\n- Improves maintainability and consistency\n- Removes redundant imports in multiple files\n\nAddresses DRY principle violation in timestamp handling.",
          "timestamp": "2025-07-02T12:13:07+09:00",
          "tree_id": "3e29d20085ab6cfc59e8a2756af283db3e927efc",
          "url": "https://github.com/skanehira/ghost/commit/ab3433751f75814351ee5d722ef6e441c412fe20"
        },
        "date": 1751426333109,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.94,
            "range": "± 0.21",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "8c60779f5979a7ec2d74bcbae823cad89f3e8663",
          "message": "chore: remove unused file",
          "timestamp": "2025-07-02T12:38:26+09:00",
          "tree_id": "3ef53b0d4e5eea5cf508234d78136e21e6c97f58",
          "url": "https://github.com/skanehira/ghost/commit/8c60779f5979a7ec2d74bcbae823cad89f3e8663"
        },
        "date": 1751427750133,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.95,
            "range": "± 0.21",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "14655948921952c103a26e006c08e45f4af5cbc2",
          "message": "refactor: unify error handling with direct enum variants\n\n- Consolidate ProcessError and StorageError into single GhostError\n- Remove unnecessary constructor methods (task_not_found, etc.)\n- Use direct enum variants: GhostError::TaskNotFound { task_id }\n- Eliminate string-to-error conversions for better type safety\n- Remove storage/error.rs module (no longer needed)\n- Update all error creation sites to use structured variants\n\nResults in more Rust-idiomatic error handling with better context.",
          "timestamp": "2025-07-02T13:28:34+09:00",
          "tree_id": "1da47ad5d92c4d68da9bcee065f0cfa8c1a66da2",
          "url": "https://github.com/skanehira/ghost/commit/14655948921952c103a26e006c08e45f4af5cbc2"
        },
        "date": 1751433643382,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.96,
            "range": "± 0.54",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "7bb6a57c456f8ebb665a277a37ca5bdacd4eece2",
          "message": "feat: implement log viewer with full keyboard navigation\n\n- Add log viewer component (LogViewerWidget) for displaying task logs\n- Implement ViewMode enum to switch between TaskList and LogView\n- Add complete keyboard navigation in log view:\n  - j/k: scroll up/down through log content\n  - g/G: jump to top/bottom of log\n  - Esc: return to task list\n  - q: quit application\n- Fix log view key binding issues with proper state management\n- Add scroll offset tracking and log line counting\n- Update footer to show 'l:Log' hint in task list\n- Add comprehensive tests for log viewer functionality\n- Update TODO.md to mark Phase 4 (log display) as completed\n\nResolves log display keyboard navigation and completes core TUI functionality.",
          "timestamp": "2025-07-02T15:35:37+09:00",
          "tree_id": "bdbe3448f0417487357c66c23214bf20bb1f8969",
          "url": "https://github.com/skanehira/ghost/commit/7bb6a57c456f8ebb665a277a37ca5bdacd4eece2"
        },
        "date": 1751438403722,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.97,
            "range": "± 0.22",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "381da9b825590a4b16eff9d923245c6b44e35208",
          "message": "feat: improve string formatting for clarity\n\nEnhanced string formatting across multiple modules for better readability\nand maintainability. Updated usage of format macros to leverage named\narguments for improved clarity.\n\nChanges include:\n- `truncate_string` in `display.rs` now uses named arguments.\n- Improved error messages in `file_watcher.rs` with named arguments.\n- Enhanced task validation error messages in `task_validation.rs`.\n- Refactored test cases in `tui_tests.rs` to use named arguments for\n  formatting strings.",
          "timestamp": "2025-07-03T01:19:12+09:00",
          "tree_id": "1ece113bc0349356b801b417193a1bc85d2a687c",
          "url": "https://github.com/skanehira/ghost/commit/381da9b825590a4b16eff9d923245c6b44e35208"
        },
        "date": 1751473266805,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.94,
            "range": "± 0.13",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "963aaff4ecea54ef2fef477fc0c48cd4f34297cf",
          "message": "fix: use inline format strings throughout codebase\n\n- Replace format\\!(\"text {}\", var) with format\\!(\"text {var}\")\n- Update println\\!, writeln\\! to use embedded expressions\n- Follow Rust formatting best practices in CLAUDE.md\n- Improve code readability and consistency",
          "timestamp": "2025-07-05T10:16:45+09:00",
          "tree_id": "9bf5ad48684ea75e30a96963905fa96bb94514bd",
          "url": "https://github.com/skanehira/ghost/commit/963aaff4ecea54ef2fef477fc0c48cd4f34297cf"
        },
        "date": 1751678393917,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.96,
            "range": "± 0.20",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "62804b6e02acdab68f15f513cd8b376c24d76f4a",
          "message": "feat: implement incremental log updates and cleanup\n\n- Add incremental file reading to only load new content\n- Implement smart update strategy (FullReload, Incremental, UseCache)\n- Track file size in cache for efficient diff updates\n- Remove unused LogViewer struct (dead code)\n- Add comprehensive tests for auto-updates and incremental loading",
          "timestamp": "2025-07-05T10:43:47+09:00",
          "tree_id": "9247a87408ada27cb4e2746c1477396a820c51ee",
          "url": "https://github.com/skanehira/ghost/commit/62804b6e02acdab68f15f513cd8b376c24d76f4a"
        },
        "date": 1751768506721,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.01,
            "range": "± 0.18",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "7ddc7901254b38203ce468678a1e9f95552f96a3",
          "message": "feat: add task termination with process group support\n\n- Add 's' key for SIGTERM (graceful stop)\n- Add Ctrl+K for SIGKILL (force stop)\n- Update footer to show s/C-k:Stop keybindings\n- Prioritize process group kill over individual PID kill\n- Fix npm/Next.js child process cleanup issue\n- Add tests for task termination functionality",
          "timestamp": "2025-07-06T12:46:28+09:00",
          "tree_id": "bc5dfebc72b29e672be87fd06ffa46458ee95c4f",
          "url": "https://github.com/skanehira/ghost/commit/7ddc7901254b38203ce468678a1e9f95552f96a3"
        },
        "date": 1751786333578,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.02,
            "range": "± 0.11",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "0e80d97b082beac6925b87a94822a27fbc2ff269",
          "message": "chore: add TODO.md",
          "timestamp": "2025-07-06T17:13:31+09:00",
          "tree_id": "f5571686c848d79ca204a3eb2e45c24d9ebff562",
          "url": "https://github.com/skanehira/ghost/commit/0e80d97b082beac6925b87a94822a27fbc2ff269"
        },
        "date": 1751789704545,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18,
            "range": "± 0.15",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "169f57b1ed1e32af1770cfd8f6c6089dad972b85",
          "message": "feat: hide scrollbars in log viewer\n\n- Use ScrollbarVisibility::Never to hide both scrollbars\n- Keep functionality with keyboard navigation (j/k/h/l)\n- Cleaner interface as keybindings are shown in footer",
          "timestamp": "2025-07-06T18:31:57+09:00",
          "tree_id": "7c4386949edc0ec84c92f5d8804e31ecd92a1040",
          "url": "https://github.com/skanehira/ghost/commit/169f57b1ed1e32af1770cfd8f6c6089dad972b85"
        },
        "date": 1751794592960,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.05,
            "range": "± 1.17",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "71134be33ee677fc495201307c9756d5f64b8e36",
          "message": "feat: show full task IDs in list command and task list UI\n\n- Display full 36-character UUIDs in list command output\n- Show as much of the UUID as possible in task list UI (limited by terminal width)\n- Update test expectations to match new task ID display format\n- Mark completed tasks in TODO.md",
          "timestamp": "2025-07-06T18:49:26+09:00",
          "tree_id": "7e4efbabe88f5d6041c84adae9c547a982cd871b",
          "url": "https://github.com/skanehira/ghost/commit/71134be33ee677fc495201307c9756d5f64b8e36"
        },
        "date": 1751795630350,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.02,
            "range": "± 0.13",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "c2b03fbc5170156a1260e8fc09fdfd96c41990c6",
          "message": "fix: resolve clippy warning about literal format string\n\n- Fix println\\! format string to include 'Directory' in the format part\n- Move 'Directory' from arguments to format string",
          "timestamp": "2025-07-06T19:04:46+09:00",
          "tree_id": "65d4f16c7a33ac44d45011bb1d3f8571abb03445",
          "url": "https://github.com/skanehira/ghost/commit/c2b03fbc5170156a1260e8fc09fdfd96c41990c6"
        },
        "date": 1751797190000,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.01,
            "range": "± 0.09",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "committer": {
            "email": "sho19921005@gmail.com",
            "name": "skanehira",
            "username": "skanehira"
          },
          "distinct": true,
          "id": "2d020eda52e892c46ac12a5edcfeb8e663256bcb",
          "message": "fix: use EventStream for non-blocking event handling in TUI\n\n- Replace blocking event::read() with async EventStream\n- Enable crossterm's event-stream feature\n- Fix issue where refresh interval was blocked by keyboard input\n- Now tasks refresh every second regardless of user input",
          "timestamp": "2025-07-06T23:06:42+09:00",
          "tree_id": "60b0ca0933b23d978a2903bb921a5af4c98b93b5",
          "url": "https://github.com/skanehira/ghost/commit/2d020eda52e892c46ac12a5edcfeb8e663256bcb"
        },
        "date": 1751811446585,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.02,
            "range": "± 0.09",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}