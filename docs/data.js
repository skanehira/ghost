window.BENCHMARK_DATA = {
  "lastUpdate": 1755305301640,
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
          "id": "445495d00871f6926506668d8c2e0f36f42a33e3",
          "message": "feat: implement Tab key to cycle through task filters in TUI\n\n- Add Tab key handler to cycle through All -> Running -> Exited -> Killed -> All\n- Reset selection when changing filters\n- Add test for Tab key filter cycling functionality\n- Update existing tests to use dynamic version from Cargo.toml",
          "timestamp": "2025-07-07T01:24:53+09:00",
          "tree_id": "1e40ed67ae5cb79e0829f1619630e5917a826725",
          "url": "https://github.com/skanehira/ghost/commit/445495d00871f6926506668d8c2e0f36f42a33e3"
        },
        "date": 1751819212020,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.01,
            "range": "± 0.14",
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
          "id": "41be6bf60c57493e6fec697d000060abab024440",
          "message": "docs: replace old design documents with current architecture documentation\n\n- Remove outdated design.md and one-shot-design.md\n- Add architecture.md with current system design and implementation details\n- Add design-decisions.md documenting key architectural choices and rationale\n- Document default data directory behavior when GHOST_DATA_DIR is not set\n- Include comprehensive documentation of components, database schema, process management, and TUI design",
          "timestamp": "2025-07-07T02:10:44+09:00",
          "tree_id": "f977f3b1d8b3e166e544ef81ebea7351c4ee1067",
          "url": "https://github.com/skanehira/ghost/commit/41be6bf60c57493e6fec697d000060abab024440"
        },
        "date": 1751821954977,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.01,
            "range": "± 0.04",
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
          "id": "49651e465dc0c7582f1a4113756441978feca901",
          "message": "fix: update TUI tests to use temporary database configuration\n\n* Modify init_database to accept optional Config parameter\n* Add init_database_with_config function for test isolation\n* Update TuiApp to use proper database initialization\n* Create TestEnvironment struct with temporary data directories\n* All TUI tests now use isolated temporary databases",
          "timestamp": "2025-07-07T02:25:51+09:00",
          "tree_id": "5b0b03024651655cfaaeb7c8225aae87a4188a1f",
          "url": "https://github.com/skanehira/ghost/commit/49651e465dc0c7582f1a4113756441978feca901"
        },
        "date": 1751822904515,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.01,
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
          "id": "bcaba62f9f86b16417120e6abd1f9de3d7255100",
          "message": "feat(ui): enhance border styling in TUI components\n\nUpdated border styles in LogViewerScrollWidget and TaskListWidget to\nimprove visual clarity. Added light magenta and green colors for\nbetter differentiation. Updated README to reflect new image paths\nand added ghost.png for documentation purposes.\n\nBREAKING CHANGE: Renamed ghost.png to images/logo.png, which may\naffect existing references.",
          "timestamp": "2025-07-07T02:40:44+09:00",
          "tree_id": "c45836970808c65b3c564b2a6342a0c3a4e5edee",
          "url": "https://github.com/skanehira/ghost/commit/bcaba62f9f86b16417120e6abd1f9de3d7255100"
        },
        "date": 1751823741087,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.03,
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
          "id": "94233cdbe5c1e383870647e3eb07dc3e358592cb",
          "message": "feat(ui): enhance border styling in TUI components\n\nUpdated border styles in LogViewerScrollWidget and TaskListWidget to\nimprove visual clarity. Added light magenta and green colors for\nbetter differentiation. Updated README to reflect new image paths\nand added ghost.png for documentation purposes.\n\nBREAKING CHANGE: Renamed ghost.png to images/logo.png, which may\naffect existing references.",
          "timestamp": "2025-07-07T02:41:46+09:00",
          "tree_id": "9a31e6472af31ab10c18849fd84270706fcb2e77",
          "url": "https://github.com/skanehira/ghost/commit/94233cdbe5c1e383870647e3eb07dc3e358592cb"
        },
        "date": 1751823801675,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.01,
            "range": "± 0.10",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "32c03e87f33abdb023ab4b58618e79f2af7f71b9",
          "message": "chore(deps): bump tokio from 1.45.1 to 1.46.1 (#5)\n\nBumps [tokio](https://github.com/tokio-rs/tokio) from 1.45.1 to 1.46.1.\n- [Release notes](https://github.com/tokio-rs/tokio/releases)\n- [Commits](https://github.com/tokio-rs/tokio/compare/tokio-1.45.1...tokio-1.46.1)\n\n---\nupdated-dependencies:\n- dependency-name: tokio\n  dependency-version: 1.46.1\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-07-07T08:28:33+09:00",
          "tree_id": "f01982f7f31bf184103740e54791d9968ef63143",
          "url": "https://github.com/skanehira/ghost/commit/32c03e87f33abdb023ab4b58618e79f2af7f71b9"
        },
        "date": 1751844602613,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.03,
            "range": "± 1.60",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8c89569d06886c62d5af520a25bcdc70862408fe",
          "message": "chore(deps): bump notify from 8.0.0 to 8.1.0 (#4)\n\nBumps [notify](https://github.com/notify-rs/notify) from 8.0.0 to 8.1.0.\n- [Release notes](https://github.com/notify-rs/notify/releases)\n- [Changelog](https://github.com/notify-rs/notify/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/notify-rs/notify/compare/notify-8.0.0...notify-8.1.0)\n\n---\nupdated-dependencies:\n- dependency-name: notify\n  dependency-version: 8.1.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-07-07T08:31:56+09:00",
          "tree_id": "d97f79d70a8bc82a7ffe15d38ba8601341d46d79",
          "url": "https://github.com/skanehira/ghost/commit/8c89569d06886c62d5af520a25bcdc70862408fe"
        },
        "date": 1751844807582,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.02,
            "range": "± 0.28",
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
          "id": "989cffa345668501d029ef9ba872030223e63d75",
          "message": "refactor: make TUI the default mode when no arguments provided\n\n- Remove the `tui` subcommand\n- Make CLI commands optional with `Option<Commands>`\n- Launch TUI mode when no arguments are provided\n- Update README to reflect the new usage pattern\n- Improve help message to indicate TUI is the default\n- Clean up unused code in process.rs (empty command check and pgid variable)\n\nThis change makes the tool more intuitive by defaulting to the interactive\nTUI mode when users run `ghost` without any arguments.",
          "timestamp": "2025-07-07T12:48:24+09:00",
          "tree_id": "35ab205f2a1fa4df1aeefedaca005c7e5ac83d87",
          "url": "https://github.com/skanehira/ghost/commit/989cffa345668501d029ef9ba872030223e63d75"
        },
        "date": 1751860274224,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.01,
            "range": "± 0.06",
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
          "id": "f3dfcab4c19907b28754a7f30abd470b37b88d39",
          "message": "chore: fmt",
          "timestamp": "2025-07-07T13:51:57+09:00",
          "tree_id": "23712aab83eff9a952672b6f47db862683d73597",
          "url": "https://github.com/skanehira/ghost/commit/f3dfcab4c19907b28754a7f30abd470b37b88d39"
        },
        "date": 1751864330566,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18,
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
          "id": "24bc80be1207b82e0b759b806934ed4366687cbf",
          "message": "feat: add page scrolling with Ctrl+d/u\n\n- Add page down/up functionality to log viewer with Ctrl+d/u\n- Add page down/up functionality to task list with Ctrl+d/u\n- Add page down/up functionality to process details env vars with Ctrl+d/u\n- Implement page_down() and page_up() methods in TableScroll\n- Calculate dynamic page size for task list based on terminal height\n- Update all footers to show C-d/u:Page keybinding\n- Add tests for page navigation functionality\n- Update all expected output files for new footer text",
          "timestamp": "2025-07-08T11:51:27+09:00",
          "tree_id": "13238487d7229d3fad74d31c5cef414cc322a4c3",
          "url": "https://github.com/skanehira/ghost/commit/24bc80be1207b82e0b759b806934ed4366687cbf"
        },
        "date": 1751943427384,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.03,
            "range": "± 0.80",
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
          "id": "2f4685991124fd600a2a8d1a2ab0a9bbf266fe62",
          "message": "fix: change rust edition to 2021 and improve cargo-cross setup",
          "timestamp": "2025-07-08T13:19:59+09:00",
          "tree_id": "4afde1868f25e08d8e0f055c321910f679fda409",
          "url": "https://github.com/skanehira/ghost/commit/2f4685991124fd600a2a8d1a2ab0a9bbf266fe62"
        },
        "date": 1751948500154,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.04,
            "range": "± 0.19",
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
          "id": "b266e66939148691aa153e437cd976928ec1afc1",
          "message": "fix: improve aarch64-linux-gnu cross-compilation setup and restore edition 2024",
          "timestamp": "2025-07-08T13:42:12+09:00",
          "tree_id": "0f1a7a1004f054031c224a5174483e52ff08dad6",
          "url": "https://github.com/skanehira/ghost/commit/b266e66939148691aa153e437cd976928ec1afc1"
        },
        "date": 1751949843310,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.02,
            "range": "± 0.07",
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
          "id": "172e4f65cd38743fc3beacf89200c933c4dc034c",
          "message": "fix: update tests for v0.1.0 and improve E2E test stability\n\n- Update version numbers in expected test files (v0.0.1 -> v0.1.0)\n- Add wait times in E2E tests to handle async process startup\n- Fix timing issues with process spawning in E2E tests\n\n🤖 Generated with [Claude Code](https://claude.ai/code)\n\nCo-Authored-By: Claude <noreply@anthropic.com>",
          "timestamp": "2025-07-13T15:07:24+09:00",
          "tree_id": "93f9249bcaaf266f7494e2743bf9a0fa7bfae975",
          "url": "https://github.com/skanehira/ghost/commit/172e4f65cd38743fc3beacf89200c933c4dc034c"
        },
        "date": 1752386952226,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.01,
            "range": "± 0.25",
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
          "id": "ad5a4011232318f7a981dd60d41908acae504682",
          "message": "[STRUCTURAL] Fix E2E test runner to properly detect and report failures\n\n- Continue running all tests even when some fail (using `|| true`)\n- Properly track and report failed tests in summary\n- Exit with code 1 when there are failures\n- Now correctly shows \"Failed: N\" instead of \"All tests passed\\!\" when tests fail\n\n🤖 Generated with [Claude Code](https://claude.ai/code)\n\nCo-Authored-By: Claude <noreply@anthropic.com>",
          "timestamp": "2025-07-13T15:29:24+09:00",
          "tree_id": "96475f17994618c9181cde248757f8af5b4b3f07",
          "url": "https://github.com/skanehira/ghost/commit/ad5a4011232318f7a981dd60d41908acae504682"
        },
        "date": 1752388808403,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.02,
            "range": "± 0.15",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "8a0389d2a250ce1c88107bbd53625bcaf1bc4bfb",
          "message": "chore(deps): bump clap from 4.5.40 to 4.5.41 (#6)\n\n---\nupdated-dependencies:\n- dependency-name: clap\n  dependency-version: 4.5.41\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-07-18T20:54:18+09:00",
          "tree_id": "9021379eccf77ee0a491e84082f9f9d402c1e43a",
          "url": "https://github.com/skanehira/ghost/commit/8a0389d2a250ce1c88107bbd53625bcaf1bc4bfb"
        },
        "date": 1752839761608,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.03,
            "range": "± 0.18",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d1719b49db805be4823778b5ffaa3324a5674f01",
          "message": "chore(deps): bump serde_json from 1.0.140 to 1.0.141 (#8)\n\nBumps [serde_json](https://github.com/serde-rs/json) from 1.0.140 to 1.0.141.\n- [Release notes](https://github.com/serde-rs/json/releases)\n- [Commits](https://github.com/serde-rs/json/compare/v1.0.140...v1.0.141)\n\n---\nupdated-dependencies:\n- dependency-name: serde_json\n  dependency-version: 1.0.141\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-07-24T20:47:27+09:00",
          "tree_id": "9da82fc39d47bbc7b1cb3a02de87600964493ac2",
          "url": "https://github.com/skanehira/ghost/commit/d1719b49db805be4823778b5ffaa3324a5674f01"
        },
        "date": 1753357751581,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.01,
            "range": "± 0.10",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "890a69747ce0dec68e760e037baa9a1df0379ec4",
          "message": "chore(deps): bump tokio from 1.46.1 to 1.47.0 (#9)\n\nBumps [tokio](https://github.com/tokio-rs/tokio) from 1.46.1 to 1.47.0.\n- [Release notes](https://github.com/tokio-rs/tokio/releases)\n- [Commits](https://github.com/tokio-rs/tokio/compare/tokio-1.46.1...tokio-1.47.0)\n\n---\nupdated-dependencies:\n- dependency-name: tokio\n  dependency-version: 1.47.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-07-31T16:58:49+09:00",
          "tree_id": "f7d89491f1bb18e1d8b13fbf20818549c3ed62a6",
          "url": "https://github.com/skanehira/ghost/commit/890a69747ce0dec68e760e037baa9a1df0379ec4"
        },
        "date": 1753948824492,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.02,
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
          "id": "f6e337899b01bbe0714c2803c9384fca08659cf0",
          "message": "feat: add listening port detection to process details view\n\n- Add port_detector module to detect listening ports using lsof\n- Update process details UI to show listening ports section\n- Add integration tests for port detection functionality\n- Fix linting issues in test files (remove unnecessary clones, fix format args)\n\n🤖 Generated with [Claude Code](https://claude.ai/code)\n\nCo-Authored-By: Claude <noreply@anthropic.com>",
          "timestamp": "2025-08-04T16:38:04+09:00",
          "tree_id": "7324e57acde8447bd59f9d8f637092982efc8415",
          "url": "https://github.com/skanehira/ghost/commit/f6e337899b01bbe0714c2803c9384fca08659cf0"
        },
        "date": 1754300218960,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.03,
            "range": "± 1.10",
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
          "id": "401cac0629374ae7d993a998368a01afb649ebf1",
          "message": "ci: fix fail",
          "timestamp": "2025-08-05T00:18:52+09:00",
          "tree_id": "e7b42d76a6124f70cf1e4ffee943a58f997a86c1",
          "url": "https://github.com/skanehira/ghost/commit/401cac0629374ae7d993a998368a01afb649ebf1"
        },
        "date": 1754320841722,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.02,
            "range": "± 0.12",
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
          "id": "983311d1f88161a28df673df78332bd55022ddb7",
          "message": "[BEHAVIORAL] Optimize lsof command checking and improve error messages\n\n- Cache lsof availability check using OnceLock to avoid repeated execution\n- Add CommandNotFound error type for better error categorization\n- Display user-friendly install messages when lsof is not available\n- Show \"lsof command not found\" and \"Please install lsof to enable port detection\" in UI\n- Improve test stability with longer wait time for TCP server startup\n\nPerformance improvements:\n- lsof availability checked only once per application run\n- Reduced system calls for port detection operations\n\nUX improvements:\n- Clear installation instructions when dependencies are missing\n- Color-coded error messages (yellow for warnings, red for errors)\n\n🤖 Generated with [Claude Code](https://claude.ai/code)\n\nCo-Authored-By: Claude <noreply@anthropic.com>",
          "timestamp": "2025-08-14T09:52:34+09:00",
          "tree_id": "e782b4a3e6998111ba56c98ad106f8c3c9f0ba53",
          "url": "https://github.com/skanehira/ghost/commit/983311d1f88161a28df673df78332bd55022ddb7"
        },
        "date": 1755132890685,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.95,
            "range": "± 0.14",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "7cb11563dc596109bb7b8123cc1899b670befe40",
          "message": "chore(deps): bump tokio from 1.47.0 to 1.47.1 (#11)\n\nBumps [tokio](https://github.com/tokio-rs/tokio) from 1.47.0 to 1.47.1.\n- [Release notes](https://github.com/tokio-rs/tokio/releases)\n- [Commits](https://github.com/tokio-rs/tokio/compare/tokio-1.47.0...tokio-1.47.1)\n\n---\nupdated-dependencies:\n- dependency-name: tokio\n  dependency-version: 1.47.1\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-08-16T09:45:43+09:00",
          "tree_id": "2a8e3fbab083fd4d5e921356f17379c52cd62b3b",
          "url": "https://github.com/skanehira/ghost/commit/7cb11563dc596109bb7b8123cc1899b670befe40"
        },
        "date": 1755305243835,
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
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "d2b9b44e2fd738f6941914ced5691a157645f52f",
          "message": "chore(deps): bump serde_json from 1.0.141 to 1.0.142 (#12)\n\nBumps [serde_json](https://github.com/serde-rs/json) from 1.0.141 to 1.0.142.\n- [Release notes](https://github.com/serde-rs/json/releases)\n- [Commits](https://github.com/serde-rs/json/compare/v1.0.141...v1.0.142)\n\n---\nupdated-dependencies:\n- dependency-name: serde_json\n  dependency-version: 1.0.142\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-08-16T09:46:02+09:00",
          "tree_id": "f588b6072e037b8f1f226af624306a09fba2339b",
          "url": "https://github.com/skanehira/ghost/commit/d2b9b44e2fd738f6941914ced5691a157645f52f"
        },
        "date": 1755305265868,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.94,
            "range": "± 0.20",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "047be3888cd7635fbfbd8a03cee5f2982dcee600",
          "message": "chore(deps): bump tokio-util from 0.7.15 to 0.7.16 (#13)\n\nBumps [tokio-util](https://github.com/tokio-rs/tokio) from 0.7.15 to 0.7.16.\n- [Release notes](https://github.com/tokio-rs/tokio/releases)\n- [Commits](https://github.com/tokio-rs/tokio/compare/tokio-util-0.7.15...tokio-util-0.7.16)\n\n---\nupdated-dependencies:\n- dependency-name: tokio-util\n  dependency-version: 0.7.16\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-08-16T09:46:13+09:00",
          "tree_id": "42249905c6e4e6c0cbd550e17841737281edb7c3",
          "url": "https://github.com/skanehira/ghost/commit/047be3888cd7635fbfbd8a03cee5f2982dcee600"
        },
        "date": 1755305279451,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.94,
            "range": "± 0.51",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "4d7424f2661a5c5a64812b03be7c3065d90f9419",
          "message": "chore(deps): bump notify from 8.1.0 to 8.2.0 (#14)\n\nBumps [notify](https://github.com/notify-rs/notify) from 8.1.0 to 8.2.0.\n- [Release notes](https://github.com/notify-rs/notify/releases)\n- [Changelog](https://github.com/notify-rs/notify/blob/main/CHANGELOG.md)\n- [Commits](https://github.com/notify-rs/notify/compare/notify-8.1.0...notify-8.2.0)\n\n---\nupdated-dependencies:\n- dependency-name: notify\n  dependency-version: 8.2.0\n  dependency-type: direct:production\n  update-type: version-update:semver-minor\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-08-16T09:46:26+09:00",
          "tree_id": "e97005fa14e25b5d4a34c9d73f6c6c5e2bf6b011",
          "url": "https://github.com/skanehira/ghost/commit/4d7424f2661a5c5a64812b03be7c3065d90f9419"
        },
        "date": 1755305288272,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.95,
            "range": "± 3.22",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "49699333+dependabot[bot]@users.noreply.github.com",
            "name": "dependabot[bot]",
            "username": "dependabot[bot]"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c22825c5f2570a1fa86eb72eeb0d9b84e55813fc",
          "message": "chore(deps): bump clap from 4.5.41 to 4.5.43 (#15)\n\nBumps [clap](https://github.com/clap-rs/clap) from 4.5.41 to 4.5.43.\n- [Release notes](https://github.com/clap-rs/clap/releases)\n- [Changelog](https://github.com/clap-rs/clap/blob/master/CHANGELOG.md)\n- [Commits](https://github.com/clap-rs/clap/compare/clap_complete-v4.5.41...clap_complete-v4.5.43)\n\n---\nupdated-dependencies:\n- dependency-name: clap\n  dependency-version: 4.5.43\n  dependency-type: direct:production\n  update-type: version-update:semver-patch\n...\n\nSigned-off-by: dependabot[bot] <support@github.com>\nCo-authored-by: dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>",
          "timestamp": "2025-08-16T09:46:41+09:00",
          "tree_id": "ead55150f69d3c137d263d6c7f20aa72ed03dd1d",
          "url": "https://github.com/skanehira/ghost/commit/c22825c5f2570a1fa86eb72eeb0d9b84e55813fc"
        },
        "date": 1755305301085,
        "tool": "cargo",
        "benches": [
          {
            "name": "tests::bench_fib",
            "value": 18.95,
            "range": "± 0.12",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}