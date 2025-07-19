use ghost::app::config::Config;
use ghost::app::storage::task::Task;
use ghost::app::storage::task_status::TaskStatus;
use ghost::app::tui::app::TuiApp;
use ghost::app::tui::ViewMode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tempfile::TempDir;
use std::fs;

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
fn test_log_auto_scroll_toggle() {
    let env = TestEnvironment::new();
    let mut app = TuiApp::new_with_config(env.config.clone()).unwrap();

    // Create test log file
    let log_dir = env._temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();
    let log_path = log_dir.join("test.log");
    fs::write(&log_path, "Initial log content\n").unwrap();

    // Add a test task
    let task = Task {
        id: "test-auto-scroll".to_string(),
        pid: 12345,
        pgid: Some(12345),
        command: r#"["echo", "test"]"#.to_string(),
        env: None,
        cwd: None,
        status: TaskStatus::Running,
        exit_code: None,
        started_at: 1704109200,
        finished_at: None,
        log_path: log_path.to_string_lossy().to_string(),
    };
    app.tasks = vec![task];
    app.table_scroll.set_total_items(1);

    // Initially auto-scroll should be false
    assert!(!app.log_auto_scroll);

    // Enter log view
    let key_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(key_enter).unwrap();
    assert_eq!(app.view_mode, ViewMode::LogView);
    
    // Auto-scroll should still be false after entering log view
    assert!(!app.log_auto_scroll);

    // Press 'f' to enable auto-scroll
    let key_f = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
    app.handle_key(key_f.clone()).unwrap();
    assert!(app.log_auto_scroll);

    // Press 'f' again to disable auto-scroll
    app.handle_key(key_f).unwrap();
    assert!(!app.log_auto_scroll);
}

#[test]
fn test_auto_scroll_disabled_by_manual_scroll() {
    let env = TestEnvironment::new();
    let mut app = TuiApp::new_with_config(env.config.clone()).unwrap();

    // Create test log file
    let log_dir = env._temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();
    let log_path = log_dir.join("test.log");
    fs::write(&log_path, "Line 1\nLine 2\nLine 3\n").unwrap();

    // Add a test task
    let task = Task {
        id: "test-manual-scroll".to_string(),
        pid: 54321,
        pgid: Some(54321),
        command: r#"["tail", "-f", "log.txt"]"#.to_string(),
        env: None,
        cwd: None,
        status: TaskStatus::Running,
        exit_code: None,
        started_at: 1704109200,
        finished_at: None,
        log_path: log_path.to_string_lossy().to_string(),
    };
    app.tasks = vec![task];
    app.table_scroll.set_total_items(1);

    // Enter log view and enable auto-scroll
    let key_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(key_enter).unwrap();
    
    let key_f = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
    app.handle_key(key_f).unwrap();
    assert!(app.log_auto_scroll);

    // Manual scroll with 'j' should disable auto-scroll
    let key_j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key(key_j).unwrap();
    assert!(!app.log_auto_scroll);

    // Re-enable auto-scroll
    let key_f = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
    app.handle_key(key_f).unwrap();
    assert!(app.log_auto_scroll);

    // Manual scroll with 'k' should disable auto-scroll
    let key_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
    app.handle_key(key_k).unwrap();
    assert!(!app.log_auto_scroll);

    // Re-enable auto-scroll
    let key_f = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
    app.handle_key(key_f).unwrap();
    assert!(app.log_auto_scroll);

    // Manual scroll with Ctrl+D should disable auto-scroll
    let key_ctrl_d = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
    app.handle_key(key_ctrl_d).unwrap();
    assert!(!app.log_auto_scroll);
}

#[test]
fn test_auto_scroll_reset_on_exit() {
    let env = TestEnvironment::new();
    let mut app = TuiApp::new_with_config(env.config.clone()).unwrap();

    // Create test log file
    let log_dir = env._temp_dir.path().join("logs");
    fs::create_dir_all(&log_dir).unwrap();
    let log_path = log_dir.join("test.log");
    fs::write(&log_path, "Test log\n").unwrap();

    // Add a test task
    let task = Task {
        id: "test-exit".to_string(),
        pid: 99999,
        pgid: Some(99999),
        command: r#"["echo", "test"]"#.to_string(),
        env: None,
        cwd: None,
        status: TaskStatus::Running,
        exit_code: None,
        started_at: 1704109200,
        finished_at: None,
        log_path: log_path.to_string_lossy().to_string(),
    };
    app.tasks = vec![task];
    app.table_scroll.set_total_items(1);

    // Enter log view and enable auto-scroll
    let key_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(key_enter).unwrap();
    
    let key_f = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
    app.handle_key(key_f).unwrap();
    assert!(app.log_auto_scroll);

    // Exit log view with 'q'
    let key_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
    app.handle_key(key_q).unwrap();
    assert_eq!(app.view_mode, ViewMode::TaskList);
    assert!(!app.log_auto_scroll); // Should be reset

    // Re-enter log view
    app.handle_key(key_enter).unwrap();
    assert_eq!(app.view_mode, ViewMode::LogView);
    assert!(!app.log_auto_scroll); // Should start as false
}