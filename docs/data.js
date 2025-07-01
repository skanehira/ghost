window.BENCHMARK_DATA = {
  "lastUpdate": 1751399547098,
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
      }
    ]
  }
}