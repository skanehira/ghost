use ghost::app::config::Config;
use ghost::app::storage::task::Task;
use ghost::app::storage::task_status::TaskStatus;
use ghost::app::tui::{App, TaskFilter, ViewMode};
use pretty_assertions::assert_eq;
use ratatui::{Terminal, backend::TestBackend};
use std::fs;
use tempfile::TempDir;

/// Helper function to load expected output from file
fn load_expected(filename: &str) -> String {
    let path = format!("tests/expected/{filename}");
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read expected file: {path}"))
}

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

/// Helper function to create test tasks
fn create_test_tasks() -> Vec<Task> {
    vec![
        Task {
            id: "abc12345-6789-1234-5678-123456789abc".to_string(),
            pid: 12345,
            pgid: Some(12345),
            command: r#"["npm","run","dev"]"#.to_string(),
            env: None,
            cwd: None,
            status: TaskStatus::Running,
            exit_code: None,
            started_at: 1704109200, // 2024-01-01 10:00 UTC
            finished_at: None,
            log_path: "/tmp/test.log".to_string(),
        },
        Task {
            id: "def67890-1234-5678-9abc-def123456789".to_string(),
            pid: 67890,
            pgid: Some(67890),
            command: r#"["cargo","build"]"#.to_string(),
            env: None,
            cwd: None,
            status: TaskStatus::Exited,
            exit_code: Some(0),
            started_at: 1704107400,        // 2024-01-01 09:30 UTC
            finished_at: Some(1704107460), // 2024-01-01 09:31 UTC
            log_path: "/tmp/test2.log".to_string(),
        },
        Task {
            id: "ghi11111-5678-9abc-def1-23456789abcd".to_string(),
            pid: 11111,
            pgid: Some(11111),
            command: r#"["python","script.py"]"#.to_string(),
            env: None,
            cwd: None,
            status: TaskStatus::Killed,
            exit_code: Some(1),
            started_at: 1704105600,        // 2024-01-01 09:00 UTC
            finished_at: Some(1704105660), // 2024-01-01 09:01 UTC
            log_path: "/tmp/test3.log".to_string(),
        },
    ]
}

/// Helper function to convert buffer to string
fn buffer_to_string(buffer: &ratatui::buffer::Buffer) -> String {
    let mut result = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let cell = &buffer[(x, y)];
            result.push_str(cell.symbol());
        }
        if y < buffer.area.height - 1 {
            result.push('\n');
        }
    }
    result
}

/// Helper function to normalize whitespace for comparison
fn normalize_buffer_output(output: &str) -> String {
    output
        .lines()
        .map(|line| line.trim_end()) // Remove trailing whitespace
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_empty_task_list_display() {
    let backend = TestBackend::new(75, 12);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app = App::new();

    terminal
        .draw(|f| {
            app.render_task_list(f, f.area());
        })
        .unwrap();

    let buffer_output = buffer_to_string(terminal.backend().buffer());
    let normalized_output = normalize_buffer_output(&buffer_output);

    let expected = load_expected("task_list_empty.txt");
    let normalized_expected = normalize_buffer_output(&expected);

    assert_eq!(
        normalized_output, normalized_expected,
        "Empty task list display does not match expected output"
    );
}

#[test]
fn test_task_list_with_tasks_display() {
    let backend = TestBackend::new(75, 12);
    let mut terminal = Terminal::new(backend).unwrap();

    let tasks = create_test_tasks();
    let mut app = App::with_tasks(tasks);

    terminal
        .draw(|f| {
            app.render_task_list(f, f.area());
        })
        .unwrap();

    let buffer_output = buffer_to_string(terminal.backend().buffer());
    let normalized_output = normalize_buffer_output(&buffer_output);

    let expected = load_expected("task_list_with_tasks.txt");
    let normalized_expected = normalize_buffer_output(&expected);

    assert_eq!(
        normalized_output, normalized_expected,
        "Task list with tasks display does not match expected output"
    );
}

#[test]
fn test_task_list_selection() {
    let backend = TestBackend::new(75, 12);
    let mut terminal = Terminal::new(backend).unwrap();

    let tasks = create_test_tasks();
    let mut app = App::with_tasks(tasks);
    app.selected_index = 1; // Select second task

    terminal
        .draw(|f| {
            app.render_task_list(f, f.area());
        })
        .unwrap();

    let buffer_output = buffer_to_string(terminal.backend().buffer());

    // Check that the output contains the tasks with truncated IDs due to column width
    // The selection highlighting will be tested once we have the expected file
    assert!(buffer_output.contains("abc123"));
    assert!(buffer_output.contains("def678"));
    assert!(buffer_output.contains("ghi111"));
}

#[test]
fn test_task_filter_display() {
    let backend = TestBackend::new(75, 8);
    let mut terminal = Terminal::new(backend).unwrap();

    let tasks = create_test_tasks();
    let mut app = App::with_tasks(tasks);
    app.filter = TaskFilter::Running;

    terminal
        .draw(|f| {
            app.render_task_list(f, f.area());
        })
        .unwrap();

    let buffer_output = buffer_to_string(terminal.backend().buffer());

    // Check that the filter is displayed in the header
    assert!(buffer_output.contains("[Filter: Running]"));
}

#[test]
fn test_footer_keybinds_display() {
    let backend = TestBackend::new(75, 12);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app = App::new();

    terminal
        .draw(|f| {
            app.render_task_list(f, f.area());
        })
        .unwrap();

    let buffer_output = buffer_to_string(terminal.backend().buffer());
    let normalized_output = normalize_buffer_output(&buffer_output);

    let expected = load_expected("task_list_empty.txt");
    let normalized_expected = normalize_buffer_output(&expected);

    assert_eq!(
        normalized_output, normalized_expected,
        "Footer keybinds display does not match expected output"
    );
}

#[test]
fn test_footer_keybinds_with_tasks() {
    let backend = TestBackend::new(75, 12);
    let mut terminal = Terminal::new(backend).unwrap();

    let tasks = create_test_tasks();
    let mut app = App::with_tasks(tasks);

    terminal
        .draw(|f| {
            app.render_task_list(f, f.area());
        })
        .unwrap();

    let buffer_output = buffer_to_string(terminal.backend().buffer());
    let normalized_output = normalize_buffer_output(&buffer_output);

    let expected = load_expected("task_list_with_tasks.txt");
    let normalized_expected = normalize_buffer_output(&expected);

    assert_eq!(
        normalized_output, normalized_expected,
        "Footer keybinds with tasks display does not match expected output"
    );
}

#[test]
fn test_footer_contains_keybinds() {
    let backend = TestBackend::new(75, 8);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut app = App::new();

    terminal
        .draw(|f| {
            app.render_task_list(f, f.area());
        })
        .unwrap();

    let buffer_output = buffer_to_string(terminal.backend().buffer());

    // Check that footer contains essential keybinds
    assert!(buffer_output.contains("j/k:Move"));
    assert!(buffer_output.contains("l:Log"));
    assert!(buffer_output.contains("s/C-k:Stop"));
    assert!(buffer_output.contains("q:Quit"));
    assert!(buffer_output.contains("g/G:Top/Bot"));
}

#[test]
fn test_task_list_vertical_layout() {
    let backend = TestBackend::new(75, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let tasks = create_test_tasks();
    let mut app = App::with_tasks(tasks);

    terminal
        .draw(|f| {
            app.render_task_list(f, f.area());
        })
        .unwrap();

    let buffer_output = buffer_to_string(terminal.backend().buffer());

    // Check that the layout has proper structure with separate blocks
    // The layout should be:
    // 1. Content block (variable height)
    // 2. Footer block (3 lines)

    let lines: Vec<&str> = buffer_output.lines().collect();

    // Content block should contain title and table
    assert!(lines[0].starts_with("┌")); // Content top border
    assert!(lines[0].contains("Ghost v"));
    assert!(lines[1].contains("ID"));
    assert!(lines[1].contains("PID"));
    assert!(lines[1].contains("Status"));

    // Footer block should be separate
    assert!(lines[lines.len() - 3].starts_with("├")); // Footer top border
    assert!(lines[lines.len() - 2].contains("j/k:Move"));
    assert!(lines[lines.len() - 2].contains("l:Log"));
    assert!(lines[lines.len() - 1].starts_with("└")); // Footer bottom border
}

#[test]
fn test_table_scroll_functionality() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ghost::app::tui::app::TuiApp;

    let env = TestEnvironment::new();
    let mut app = TuiApp::new_with_config(env.config.clone()).unwrap();

    // Create many tasks to enable scrolling
    let mut tasks = Vec::new();
    for i in 0..20 {
        tasks.push(Task {
            id: format!("task_{i:03}"),
            pid: 1000 + i as u32,
            pgid: Some(1000 + i),
            command: format!(r#"["echo","task_{i}"]"#),
            env: None,
            cwd: None,
            status: TaskStatus::Running,
            exit_code: None,
            started_at: 1704109200 + i as i64,
            finished_at: None,
            log_path: format!("/tmp/test_{i}.log"),
        });
    }
    app.tasks = tasks;
    app.table_scroll.set_total_items(20);
    app.set_selected_index(0);

    // Test scrolling down
    let key_j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key(key_j).unwrap();

    // After first j, selection should move but scroll should not change yet
    assert_eq!(app.selected_index(), 1);
    assert_eq!(app.table_scroll_offset(), 0);

    // Test scrolling up
    let key_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
    app.handle_key(key_k).unwrap();
    assert_eq!(app.selected_index(), 0);
    assert_eq!(app.table_scroll_offset(), 0);

    // Test going to bottom triggers scroll
    let key_shift_g = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
    app.handle_key(key_shift_g).unwrap();
    assert_eq!(app.selected_index(), 19); // Last task
    // Scroll offset should be adjusted to show the selected item
    assert!(app.table_scroll_offset() > 0);

    // Test going to top resets scroll
    let key_g = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    app.handle_key(key_g).unwrap();
    assert_eq!(app.selected_index(), 0);
    assert_eq!(app.table_scroll_offset(), 0);
}

#[test]
fn test_table_scroll_display() {
    let backend = TestBackend::new(75, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    // Create many tasks
    let mut tasks = Vec::new();
    for i in 0..15 {
        tasks.push(Task {
            id: format!("task_{i:03}"),
            pid: 1000 + i as u32,
            pgid: Some(1000 + i),
            command: format!(r#"["echo","task_{i}"]"#),
            env: None,
            cwd: None,
            status: TaskStatus::Running,
            exit_code: None,
            started_at: 1704109200 + i as i64,
            finished_at: None,
            log_path: format!("/tmp/test_{i}.log"),
        });
    }

    let mut app = App::with_tasks_and_scroll(tasks, 5); // Start scrolled down 5 rows

    terminal
        .draw(|f| {
            app.render_task_list(f, f.area());
        })
        .unwrap();

    let buffer_output = buffer_to_string(terminal.backend().buffer());

    // Should show tasks starting from around task_005 due to scroll offset
    // With full task IDs, the first visible task should be task_005 or later
    assert!(buffer_output.contains("task_"));
    // Should not show first few tasks due to scrolling
    // Check that task_000 is not visible (it would be scrolled out of view)
    assert!(!buffer_output.contains(" task_000 "));
}

#[test]
fn test_task_termination_keys() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ghost::app::storage::task_status::TaskStatus;
    use ghost::app::tui::app::TuiApp;

    let env = TestEnvironment::new();
    let mut app = TuiApp::new_with_config(env.config.clone()).unwrap();

    // Add a running task
    let tasks = vec![Task {
        id: "test_task".to_string(),
        pid: 12345,
        pgid: Some(12345),
        command: r#"["echo","test"]"#.to_string(),
        env: None,
        cwd: None,
        status: TaskStatus::Running,
        exit_code: None,
        started_at: 1704109200,
        finished_at: None,
        log_path: "/tmp/test.log".to_string(),
    }];
    app.tasks = tasks;
    app.table_scroll.set_total_items(1);
    app.set_selected_index(0);

    // Test 's' key for SIGTERM
    let key_s = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
    // We can't actually test the signal sending, but we can test that the handler is called
    let result = app.handle_key(key_s);
    assert!(result.is_ok());

    // Test Ctrl+K for SIGKILL
    let key_ctrl_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
    let result = app.handle_key(key_ctrl_k);
    assert!(result.is_ok());
}

#[test]
fn test_task_filter_cycling_with_tab() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ghost::app::tui::app::TuiApp;

    let env = TestEnvironment::new();
    let mut app = TuiApp::new_with_config(env.config.clone()).unwrap();

    // Add tasks with different statuses
    let tasks = vec![
        Task {
            id: "running_task".to_string(),
            pid: 12345,
            pgid: Some(12345),
            command: r#"["echo","running"]"#.to_string(),
            env: None,
            cwd: None,
            status: TaskStatus::Running,
            exit_code: None,
            started_at: 1704109200,
            finished_at: None,
            log_path: "/tmp/running.log".to_string(),
        },
        Task {
            id: "exited_task".to_string(),
            pid: 12346,
            pgid: Some(12346),
            command: r#"["echo","exited"]"#.to_string(),
            env: None,
            cwd: None,
            status: TaskStatus::Exited,
            exit_code: Some(0),
            started_at: 1704109200,
            finished_at: Some(1704109260),
            log_path: "/tmp/exited.log".to_string(),
        },
        Task {
            id: "killed_task".to_string(),
            pid: 12347,
            pgid: Some(12347),
            command: r#"["echo","killed"]"#.to_string(),
            env: None,
            cwd: None,
            status: TaskStatus::Killed,
            exit_code: Some(1),
            started_at: 1704109200,
            finished_at: Some(1704109260),
            log_path: "/tmp/killed.log".to_string(),
        },
    ];
    app.tasks = tasks;
    app.table_scroll.set_total_items(3);

    // Test initial filter is All
    assert_eq!(app.filter, TaskFilter::All);

    // Press Tab to cycle to Running
    let key_tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    app.handle_key(key_tab.clone()).unwrap();
    assert_eq!(app.filter, TaskFilter::Running);

    // Press Tab to cycle to Exited
    app.handle_key(key_tab.clone()).unwrap();
    assert_eq!(app.filter, TaskFilter::Exited);

    // Press Tab to cycle to Killed
    app.handle_key(key_tab.clone()).unwrap();
    assert_eq!(app.filter, TaskFilter::Killed);

    // Press Tab to cycle back to All
    app.handle_key(key_tab).unwrap();
    assert_eq!(app.filter, TaskFilter::All);
}

#[test]
fn test_process_details_navigation() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ghost::app::tui::app::TuiApp;

    let env = TestEnvironment::new();
    let mut app = TuiApp::new_with_config(env.config.clone()).unwrap();

    // Add a test task with environment variables
    let tasks = vec![Task {
        id: "12345678-1234-1234-1234-123456789012".to_string(),
        pid: 1234,
        pgid: Some(1234),
        command: r#"["npm", "run", "dev"]"#.to_string(),
        env: Some(r#"[["NODE_ENV","development"],["PORT","3000"]]"#.to_string()),
        cwd: Some("/home/user/project".to_string()),
        status: TaskStatus::Running,
        exit_code: None,
        started_at: 1000000000,
        finished_at: None,
        log_path: "/tmp/ghost/logs/12345678.log".to_string(),
    }];
    app.tasks = tasks;
    app.table_scroll.set_total_items(1);

    // Initial view should be TaskList
    assert_eq!(app.view_mode, ViewMode::TaskList);

    // Press Enter to view process details
    let key_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    app.handle_key(key_enter).unwrap();
    assert_eq!(app.view_mode, ViewMode::ProcessDetails);
    assert_eq!(
        app.selected_task_id,
        Some("12345678-1234-1234-1234-123456789012".to_string())
    );

    // Press Esc to go back to task list
    let key_esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key(key_esc).unwrap();
    assert_eq!(app.view_mode, ViewMode::TaskList);
    assert!(!app.should_quit());

    // Go back to process details and test q key
    app.handle_key(key_enter).unwrap();
    assert_eq!(app.view_mode, ViewMode::ProcessDetails);

    // Press q to quit
    let key_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
    app.handle_key(key_q).unwrap();
    assert!(app.should_quit());
}

#[test]
fn test_process_details_display() {
    use ghost::app::tui::app::TuiApp;

    let env = TestEnvironment::new();
    let mut app = TuiApp::new_with_config(env.config.clone()).unwrap();

    // Add a test task
    let tasks = vec![Task {
        id: "test-task-id".to_string(),
        pid: 5678,
        pgid: Some(5678),
        command: r#"["echo", "hello world"]"#.to_string(),
        env: Some(r#"[["TEST_VAR","test_value"]]"#.to_string()),
        cwd: Some("/tmp/test".to_string()),
        status: TaskStatus::Exited,
        exit_code: Some(0),
        started_at: 1000000000,
        finished_at: Some(1000001000),
        log_path: "/tmp/ghost/logs/test.log".to_string(),
    }];
    app.tasks = tasks;
    app.table_scroll.set_total_items(1);
    app.view_mode = ViewMode::ProcessDetails;
    app.selected_task_id = Some("test-task-id".to_string());

    // Create a terminal and render the process details view
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| app.render(f)).unwrap();

    // Get the buffer content
    let buffer = terminal.backend().buffer();
    let area = buffer.area();
    let content: Vec<String> = (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buffer.cell((x, y)).unwrap().symbol().to_string())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect();

    // Verify key elements are displayed
    let full_content = content.join("\n");

    // Debug output
    if !full_content.contains("Process Details") {
        eprintln!("Full content:\n{}", full_content);
    }

    assert!(full_content.contains("Process Details"));
    assert!(full_content.contains("Task ID: test-task-id"));
    assert!(full_content.contains("Command: echo hello world"));
    assert!(full_content.contains("Status: exited") || full_content.contains("Status: Exited"));
    assert!(full_content.contains("PID: 5678"));
    assert!(full_content.contains("PGID: 5678"));
    assert!(full_content.contains("Directory: /tmp/test"));
    assert!(full_content.contains("Environment Variables"));
    assert!(full_content.contains("TEST_VAR=test_value"));
    assert!(full_content.contains("[q] Quit"));
    assert!(full_content.contains("[Esc] Back to list"));
    assert!(full_content.contains("[j/k] Scroll env vars"));
}
