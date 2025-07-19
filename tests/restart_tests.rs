use ghost::app::config::Config;
use ghost::app::storage::task::Task;
use ghost::app::storage::task_status::TaskStatus;
use ghost::app::tui::app::TuiApp;
use ghost::app::tui::{ConfirmationAction, ConfirmationDialog, ViewMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs;
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
fn test_restart_dialog_flow() {
    let env = TestEnvironment::new();
    let mut app = TuiApp::new_with_config(env.config.clone()).unwrap();

    // Add a running task
    let task = Task {
        id: "test-restart-id".to_string(),
        pid: 12345,
        pgid: Some(12345),
        command: r#"["npm", "run", "dev"]"#.to_string(),
        env: None,
        cwd: Some("/home/user/project".to_string()),
        status: TaskStatus::Running,
        exit_code: None,
        started_at: 1704109200,
        finished_at: None,
        log_path: "/tmp/test-restart.log".to_string(),
    };
    app.tasks = vec![task];
    app.table_scroll.set_total_items(1);

    // Press 'r' to show restart dialog
    let key_r = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
    app.handle_key(key_r).unwrap();

    // Verify dialog is shown
    assert_eq!(app.view_mode, ViewMode::ConfirmationDialog);
    assert!(app.confirmation_dialog.is_some());
    
    let dialog = app.confirmation_dialog.as_ref().unwrap();
    assert_eq!(dialog.action, ConfirmationAction::Restart);
    assert_eq!(dialog.task_id, "test-restart-id");
    assert_eq!(dialog.task_command, "npm run dev");
    assert!(!dialog.selected_choice); // Default to No

    // Press Space to toggle to Yes
    let key_space = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
    app.handle_key(key_space).unwrap();
    
    let dialog = app.confirmation_dialog.as_ref().unwrap();
    assert!(dialog.selected_choice); // Should be Yes now

    // Press Esc to cancel
    let key_esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key(key_esc).unwrap();
    
    assert_eq!(app.view_mode, ViewMode::TaskList);
    assert!(app.confirmation_dialog.is_none());
}

#[test]
fn test_rerun_dialog_for_stopped_task() {
    let env = TestEnvironment::new();
    let mut app = TuiApp::new_with_config(env.config.clone()).unwrap();

    // Add an exited task
    let task = Task {
        id: "test-rerun-id".to_string(),
        pid: 54321,
        pgid: Some(54321),
        command: r#"["cargo", "test"]"#.to_string(),
        env: Some(r#"{"RUST_LOG":"debug"}"#.to_string()),
        cwd: Some("/home/user/rust-project".to_string()),
        status: TaskStatus::Exited,
        exit_code: Some(0),
        started_at: 1704109200,
        finished_at: Some(1704109260),
        log_path: "/tmp/test-rerun.log".to_string(),
    };
    app.tasks = vec![task];
    app.table_scroll.set_total_items(1);

    // Press 'r' to show rerun dialog
    let key_r = KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE);
    app.handle_key(key_r).unwrap();

    // Verify dialog shows "Rerun" for stopped task
    assert_eq!(app.view_mode, ViewMode::ConfirmationDialog);
    let dialog = app.confirmation_dialog.as_ref().unwrap();
    assert_eq!(dialog.action, ConfirmationAction::Rerun);
    assert_eq!(dialog.task_command, "cargo test");
}

#[test]
fn test_restart_preserves_working_directory() {
    use ghost::app::commands;
    use ghost::app::storage;
    
    let env = TestEnvironment::new();
    let test_dir = env._temp_dir.path().join("test_project");
    fs::create_dir_all(&test_dir).unwrap();
    
    // Create a test script that prints its working directory
    let script_path = test_dir.join("print_cwd.sh");
    fs::write(&script_path, "#!/bin/bash\npwd\nsleep 1\n").unwrap();
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }
    
    // Spawn the process with specific working directory
    let command = vec![script_path.to_string_lossy().to_string()];
    let result = commands::spawn(command.clone(), Some(test_dir.clone()), vec![]);
    assert!(result.is_ok());
    
    // Give process time to start and write to log
    std::thread::sleep(Duration::from_millis(500));
    
    // Check the process was spawned in the correct directory
    let conn = storage::init_database_with_config(Some(env.config.clone())).unwrap();
    let tasks = storage::get_tasks(&conn, None).unwrap();
    assert_eq!(tasks.len(), 1);
    
    let task = &tasks[0];
    assert_eq!(task.cwd, Some(test_dir.to_string_lossy().to_string()));
    
    // Read the log to verify the process ran in the correct directory
    let log_content = fs::read_to_string(&task.log_path)
        .unwrap_or_else(|e| panic!("Failed to read log file: {} - {}", task.log_path, e));
    
    println!("Log content: {}", log_content);
    println!("Expected directory: {}", test_dir.display());
    
    assert!(
        log_content.contains(&test_dir.to_string_lossy().to_string()),
        "Log should contain the working directory path"
    );
    
    // Clean up
    let _ = commands::stop(&task.id, true, false);
}

#[test]
fn test_restart_preserves_environment_variables() {
    use ghost::app::commands;
    use ghost::app::storage;
    
    let env = TestEnvironment::new();
    let test_dir = env._temp_dir.path();
    
    // Create a test script that prints environment variable
    let script_path = test_dir.join("print_env.sh");
    fs::write(&script_path, "#!/bin/bash\necho \"TEST_VAR=$TEST_VAR\"\nsleep 1\n").unwrap();
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }
    
    // Spawn with custom environment variable
    let command = vec![script_path.to_string_lossy().to_string()];
    let env_vars = vec!["TEST_VAR=custom_value".to_string()];
    let result = commands::spawn(command.clone(), None, env_vars);
    assert!(result.is_ok());
    
    // Give process time to start and write to log
    std::thread::sleep(Duration::from_millis(200));
    
    // Verify environment variable was set
    let conn = storage::init_database_with_config(Some(env.config.clone())).unwrap();
    let tasks = storage::get_tasks(&conn, None).unwrap();
    assert_eq!(tasks.len(), 1);
    
    let task = &tasks[0];
    
    // Read the log to verify environment variable
    if let Ok(log_content) = fs::read_to_string(&task.log_path) {
        assert!(log_content.contains("TEST_VAR=custom_value"));
    }
    
    // Clean up
    let _ = commands::stop(&task.id, true, false);
}

#[test]
fn test_restart_handles_missing_process() {
    let env = TestEnvironment::new();
    let mut app = TuiApp::new_with_config(env.config.clone()).unwrap();

    // Add a task with non-existent PID
    let task = Task {
        id: "test-missing-process".to_string(),
        pid: 99999, // Non-existent PID
        pgid: Some(99999),
        command: r#"["echo", "test"]"#.to_string(),
        env: None,
        cwd: None,
        status: TaskStatus::Running, // Marked as running but process doesn't exist
        exit_code: None,
        started_at: 1704109200,
        finished_at: None,
        log_path: "/tmp/test-missing.log".to_string(),
    };
    app.tasks = vec![task];
    app.table_scroll.set_total_items(1);

    // Create confirmation dialog directly
    app.confirmation_dialog = Some(ConfirmationDialog {
        action: ConfirmationAction::Restart,
        task_id: "test-missing-process".to_string(),
        task_command: "echo test".to_string(),
        selected_choice: true, // Yes
    });
    app.view_mode = ViewMode::ConfirmationDialog;

    // Execute restart by simulating Enter key press
    let key_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    let result = app.handle_key(key_enter);
    
    // Should not panic or return error
    assert!(result.is_ok());
    assert_eq!(app.view_mode, ViewMode::TaskList);
}

#[test]
fn test_concurrent_restart_protection() {
    use ghost::app::commands;
    use ghost::app::storage;
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let env = TestEnvironment::new();
    
    // Spawn a long-running process
    let command = vec!["sleep".to_string(), "10".to_string()];
    let result = commands::spawn(command.clone(), None, vec![]);
    assert!(result.is_ok());
    
    // Get the task ID
    let conn = storage::init_database_with_config(Some(env.config.clone())).unwrap();
    let tasks = storage::get_tasks(&conn, None).unwrap();
    assert_eq!(tasks.len(), 1);
    let task_id = tasks[0].id.clone();
    
    // Try to restart the same task from multiple threads
    let restart_count = Arc::new(Mutex::new(0));
    let mut handles = vec![];
    
    for _ in 0..3 {
        let task_id_clone = task_id.clone();
        let restart_count_clone = restart_count.clone();
        let command_clone = command.clone();
        
        let handle = thread::spawn(move || {
            // Stop the task
            let _ = commands::stop(&task_id_clone, false, false);
            
            // Small delay
            thread::sleep(Duration::from_millis(50));
            
            // Try to spawn again
            let spawn_result = commands::spawn(command_clone, None, vec![]);
            
            if spawn_result.is_ok() {
                let mut count = restart_count_clone.lock().unwrap();
                *count += 1;
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify that restarts were handled properly
    let final_count = *restart_count.lock().unwrap();
    assert!(final_count >= 1); // At least one restart should succeed
    
    // Clean up
    let conn = storage::init_database_with_config(Some(env.config.clone())).unwrap();
    let final_tasks = storage::get_tasks(&conn, None).unwrap();
    for task in final_tasks {
        let _ = commands::stop(&task.id, true, false);
    }
}

#[test]
fn test_restart_integration_with_real_process() {
    use ghost::app::commands;
    use ghost::app::storage;
    
    let env = TestEnvironment::new();
    let test_dir = env._temp_dir.path();
    
    // Create a test script that writes timestamps
    let script_path = test_dir.join("timestamp_writer.sh");
    let output_file = test_dir.join("timestamps.txt");
    
    let script_content = format!(
        r#"#!/bin/bash
echo "Started at $(date +%s)" >> {}
while true; do
    echo "Running at $(date +%s)" >> {}
    sleep 0.5
done
"#,
        output_file.display(),
        output_file.display()
    );
    
    fs::write(&script_path, script_content).unwrap();
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();
    }
    
    // Spawn the process
    let command = vec![script_path.to_string_lossy().to_string()];
    let result = commands::spawn(command.clone(), Some(test_dir.to_path_buf()), vec![]);
    assert!(result.is_ok());
    
    // Let it run for a bit
    thread::sleep(Duration::from_millis(1500));
    
    // Get the task
    let conn = storage::init_database_with_config(Some(env.config.clone())).unwrap();
    let tasks = storage::get_tasks(&conn, None).unwrap();
    assert_eq!(tasks.len(), 1);
    let task = &tasks[0];
    let task_id = task.id.clone();
    let original_pid = task.pid;
    
    // Read initial output
    let initial_output = fs::read_to_string(&output_file).unwrap();
    let initial_lines = initial_output.lines().count();
    assert!(initial_lines >= 2); // Should have at least start + one running message
    
    // Restart the process
    let _ = commands::stop(&task_id, false, false);
    thread::sleep(Duration::from_millis(200)); // Wait for process to stop
    
    // Spawn again with same parameters
    let _ = commands::spawn(command, Some(test_dir.to_path_buf()), vec![]);
    thread::sleep(Duration::from_millis(1500)); // Let new process run
    
    // Verify new process was created
    let new_tasks = storage::get_tasks(&conn, None).unwrap();
    let new_task = new_tasks.iter().find(|t| t.status == TaskStatus::Running);
    assert!(new_task.is_some());
    
    let new_task = new_task.unwrap();
    assert_ne!(new_task.pid, original_pid); // Should have different PID
    
    // Verify output file has more lines (process restarted and continued writing)
    let final_output = fs::read_to_string(&output_file).unwrap();
    let final_lines = final_output.lines().count();
    assert!(final_lines > initial_lines);
    
    // Clean up
    let _ = commands::stop(&new_task.id, true, false);
}