use ghost::app::commands;
use ghost::app::config::Config;
use ghost::app::storage;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Helper struct to manage test environment with temporary data directory
struct TestEnvironment {
    _temp_dir: TempDir,
    original_env: Option<String>,
    pub config: Config,
}

impl TestEnvironment {
    fn new() -> Self {
        // Save original GHOST_DATA_DIR if set
        let original_env = std::env::var("GHOST_DATA_DIR").ok();

        // Create temporary directory for test data
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Set GHOST_DATA_DIR to temp directory
        unsafe {
            std::env::set_var("GHOST_DATA_DIR", temp_dir.path());
        }

        let config = Config::with_data_dir(temp_dir.path().to_path_buf());

        Self {
            _temp_dir: temp_dir,
            original_env,
            config,
        }
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // Restore original GHOST_DATA_DIR or remove it
        unsafe {
            match &self.original_env {
                Some(val) => std::env::set_var("GHOST_DATA_DIR", val),
                None => std::env::remove_var("GHOST_DATA_DIR"),
            }
        }
    }
}

#[test]
fn test_command_not_double_wrapped_on_restart() {
    let env = TestEnvironment::new();

    // First spawn with a simple command
    let original_command = vec!["echo".to_string(), "hello world".to_string()];
    let result = commands::spawn(original_command.clone(), None, vec![]);
    assert!(result.is_ok());

    // Give process time to start
    thread::sleep(Duration::from_millis(500));

    // Get the task from database
    let conn = storage::init_database_with_config(Some(env.config.clone())).unwrap();
    let tasks = storage::get_tasks(&conn, None).unwrap();
    assert_eq!(tasks.len(), 1);

    let task = &tasks[0];
    let task_id = task.id.clone();

    // Verify the command stored in DB is the original command
    let stored_command: Vec<String> = serde_json::from_str(&task.command).unwrap();
    assert_eq!(stored_command, original_command);

    // Stop the task
    let _ = commands::stop(&task_id, true, false);
    thread::sleep(Duration::from_millis(200));

    // Restart by spawning with the stored command
    let restart_result =
        commands::spawn(stored_command, task.cwd.clone().map(|c| c.into()), vec![]);
    assert!(restart_result.is_ok());

    // Give new process time to start
    thread::sleep(Duration::from_millis(500));

    // Get all tasks (should have 2 now)
    let all_tasks = storage::get_tasks(&conn, None).unwrap();

    // Find the new running task
    let new_task = all_tasks
        .iter()
        .find(|t| t.status == storage::task_status::TaskStatus::Running)
        .expect("Should have a running task");

    // Verify the new task also has the original command
    let new_stored_command: Vec<String> = serde_json::from_str(&new_task.command).unwrap();
    assert_eq!(new_stored_command, original_command);

    // Clean up
    let _ = commands::stop(&new_task.id, true, false);
}

#[test]
fn test_complex_command_preserved() {
    let env = TestEnvironment::new();

    // Complex command with arguments and flags
    let complex_command = vec![
        "npx".to_string(),
        "slidev".to_string(),
        "--open".to_string(),
        "presentation.md".to_string(),
    ];

    let result = commands::spawn(complex_command.clone(), None, vec![]);
    assert!(result.is_ok());

    // Give process time to start
    thread::sleep(Duration::from_millis(500));

    // Verify command in database
    let conn = storage::init_database_with_config(Some(env.config.clone())).unwrap();
    let tasks = storage::get_tasks(&conn, None).unwrap();
    assert_eq!(tasks.len(), 1);

    let stored_command: Vec<String> = serde_json::from_str(&tasks[0].command).unwrap();
    assert_eq!(stored_command, complex_command);

    // Clean up
    let _ = commands::stop(&tasks[0].id, true, false);
}

#[test]
fn test_process_start_verification() {
    let env = TestEnvironment::new();

    // Try to spawn a command that exits immediately with non-interactive shell
    let failing_command = vec![
        "false".to_string(), // This command always exits with status 1
    ];

    // Even though the shell starts, the command should be marked as failed
    // because our verification should detect the quick exit
    let _result = commands::spawn(failing_command, None, vec![]);

    // Give time for process verification
    thread::sleep(Duration::from_millis(500));

    // Check the database for task status
    let conn = storage::init_database_with_config(Some(env.config.clone())).unwrap();
    let tasks = storage::get_tasks(&conn, None).unwrap();

    // We should have at least one task
    assert!(!tasks.is_empty());

    // The task might still show as running if shell is still alive,
    // but let's verify it doesn't stay running for long
    thread::sleep(Duration::from_secs(3));

    // Update status and check again
    let updated_tasks = storage::get_tasks_with_process_check(&conn, None, true).unwrap();
    if !updated_tasks.is_empty() {
        // Process should have exited by now
        assert_ne!(
            updated_tasks[0].status,
            storage::task_status::TaskStatus::Running
        );
    }
}

#[test]
fn test_working_directory_preserved_on_restart() {
    let env = TestEnvironment::new();
    let test_dir = TempDir::new().unwrap();
    let test_path = test_dir.path().to_path_buf();

    // Create a test script that prints working directory
    let script_path = test_path.join("print_pwd.sh");
    std::fs::write(&script_path, "#!/bin/bash\npwd\nsleep 2\n").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms).unwrap();
    }

    // Spawn with specific working directory
    let command = vec![script_path.to_string_lossy().to_string()];
    let result = commands::spawn(command.clone(), Some(test_path.clone()), vec![]);
    assert!(result.is_ok());

    thread::sleep(Duration::from_millis(500));

    // Get task and verify working directory
    let conn = storage::init_database_with_config(Some(env.config.clone())).unwrap();
    let tasks = storage::get_tasks(&conn, None).unwrap();
    assert_eq!(tasks.len(), 1);

    let task = &tasks[0];
    assert_eq!(task.cwd, Some(test_path.to_string_lossy().to_string()));

    // Stop and restart
    let _ = commands::stop(&task.id, true, false);
    thread::sleep(Duration::from_millis(200));

    // Restart with same working directory
    let restart_result = commands::spawn(command, task.cwd.clone().map(|c| c.into()), vec![]);
    assert!(restart_result.is_ok());

    thread::sleep(Duration::from_millis(500));

    // Verify new task has same working directory
    let all_tasks = storage::get_tasks(&conn, None).unwrap();
    let new_task = all_tasks
        .iter()
        .find(|t| t.status == storage::task_status::TaskStatus::Running)
        .expect("Should have a running task");

    assert_eq!(new_task.cwd, Some(test_path.to_string_lossy().to_string()));

    // Clean up
    let _ = commands::stop(&new_task.id, true, false);
}
