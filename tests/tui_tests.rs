use ghost::app::storage::task::Task;
use ghost::app::storage::task_status::TaskStatus;
use ghost::app::tui::{App, TaskFilter};
use pretty_assertions::assert_eq;
use ratatui::{Terminal, backend::TestBackend};
use std::fs;

/// Helper function to load expected output from file
fn load_expected(filename: &str) -> String {
    let path = format!("tests/expected/{filename}");
    fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read expected file: {path}"))
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
    assert!(buffer_output.contains("abc12345-6789-1"));
    assert!(buffer_output.contains("def67890-1234-5"));
    assert!(buffer_output.contains("ghi11111-5678-9"));
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
    assert!(lines[0].contains("Ghost TUI v0.0.1"));
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

    let mut app = TuiApp::new().unwrap();

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

    let mut app = TuiApp::new().unwrap();

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
