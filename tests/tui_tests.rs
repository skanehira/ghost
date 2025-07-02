use ghost::app::storage::task::Task;
use ghost::app::storage::task_status::TaskStatus;
use ghost::app::tui::{App, TaskFilter};
use ratatui::{Terminal, backend::TestBackend};
use std::fs;

/// Helper function to load expected output from file
fn load_expected(filename: &str) -> String {
    let path = format!("tests/expected/{}", filename);
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read expected file: {}", path))
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
    let backend = TestBackend::new(75, 8);
    let mut terminal = Terminal::new(backend).unwrap();

    let app = App::new();

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
    let backend = TestBackend::new(75, 8);
    let mut terminal = Terminal::new(backend).unwrap();

    let tasks = create_test_tasks();
    let app = App::with_tasks(tasks);

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
    let backend = TestBackend::new(75, 8);
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

    // For now, just check that the output contains the tasks (8 char IDs)
    // The selection highlighting will be tested once we have the expected file
    assert!(buffer_output.contains("abc1234"));
    assert!(buffer_output.contains("def6789"));
    assert!(buffer_output.contains("ghi1111"));
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
    let backend = TestBackend::new(75, 8);
    let mut terminal = Terminal::new(backend).unwrap();

    let app = App::new();

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
    let backend = TestBackend::new(75, 8);
    let mut terminal = Terminal::new(backend).unwrap();

    let tasks = create_test_tasks();
    let app = App::with_tasks(tasks);

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

    let app = App::new();

    terminal
        .draw(|f| {
            app.render_task_list(f, f.area());
        })
        .unwrap();

    let buffer_output = buffer_to_string(terminal.backend().buffer());

    // Check that footer contains essential keybinds
    assert!(buffer_output.contains("j/k:Move"));
    assert!(buffer_output.contains("l:Log"));
    assert!(buffer_output.contains("q:Quit"));
    assert!(buffer_output.contains("g/G:Top/Bottom"));
}

#[test]
fn test_log_viewer_display() {
    use ghost::app::tui::log_viewer::LogViewerWidget;
    use ratatui::{Terminal, backend::TestBackend};

    let backend = TestBackend::new(75, 15);
    let mut terminal = Terminal::new(backend).unwrap();

    let tasks = create_test_tasks();
    let selected_task = &tasks[0]; // First task

    terminal
        .draw(|f| {
            let widget = LogViewerWidget::new(selected_task);
            f.render_widget(widget, f.area());
        })
        .unwrap();

    let buffer_output = buffer_to_string(terminal.backend().buffer());

    // Check that log viewer header is present
    assert!(buffer_output.contains("Log Viewer"));
    assert!(buffer_output.contains("abc1234")); // Task ID should be in header

    // Check that footer keybinds are present
    assert!(buffer_output.contains("j/k:Scroll"));
    assert!(buffer_output.contains("gg/G:Top/Bottom"));
    assert!(buffer_output.contains("Esc:Back"));
}

#[test]
fn test_log_view_key_handling() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use ghost::app::tui::app::TuiApp;

    let mut app = TuiApp::new().unwrap();

    // Add test tasks
    let tasks = create_test_tasks();
    app.tasks = tasks;
    app.selected_index = 0;

    // Switch to log view
    let key_l = KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE);
    app.handle_key(key_l).unwrap();

    // Verify we're in log view mode
    assert_eq!(app.view_mode, ghost::app::tui::ViewMode::LogView);

    // Test scroll down
    let key_j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key(key_j).unwrap();

    // Test scroll up
    let key_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
    app.handle_key(key_k).unwrap();

    // Test go to top
    let key_g = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    app.handle_key(key_g).unwrap();
    assert_eq!(app.log_scroll_offset, 0);

    // Test go to bottom
    let key_shift_g = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
    app.handle_key(key_shift_g).unwrap();

    // Test return to task list
    let key_esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    app.handle_key(key_esc).unwrap();
    assert_eq!(app.view_mode, ghost::app::tui::ViewMode::TaskList);
}
