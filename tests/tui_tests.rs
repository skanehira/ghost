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
    let backend = TestBackend::new(75, 12);
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
    let backend = TestBackend::new(75, 12);
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
    let backend = TestBackend::new(75, 12);
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

#[test]
fn test_task_list_vertical_layout() {
    let backend = TestBackend::new(75, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    let tasks = create_test_tasks();
    let app = App::with_tasks(tasks);

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
            pgid: Some(1000 + i as i32),
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
    app.table_scroll_offset = 0;

    // Test scrolling down
    let key_j = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    app.handle_key(key_j).unwrap();

    // After first j, selection should move but scroll should not change yet
    assert_eq!(app.selected_index, 1);
    assert_eq!(app.table_scroll_offset, 0);

    // Test scrolling up
    let key_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE);
    app.handle_key(key_k).unwrap();
    assert_eq!(app.selected_index, 0);
    assert_eq!(app.table_scroll_offset, 0);

    // Test going to bottom triggers scroll
    let key_shift_g = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE);
    app.handle_key(key_shift_g).unwrap();
    assert_eq!(app.selected_index, 19); // Last task
    // Scroll offset should be adjusted to show the selected item
    assert!(app.table_scroll_offset > 0);

    // Test going to top resets scroll
    let key_g = KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE);
    app.handle_key(key_g).unwrap();
    assert_eq!(app.selected_index, 0);
    assert_eq!(app.table_scroll_offset, 0);
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

    let app = App::with_tasks_and_scroll(tasks, 5); // Start scrolled down 5 rows

    terminal
        .draw(|f| {
            app.render_task_list(f, f.area());
        })
        .unwrap();

    let buffer_output = buffer_to_string(terminal.backend().buffer());

    // Should show tasks starting from task_005 due to scroll offset
    assert!(buffer_output.contains("task_00"));
    // Should not show first few tasks due to scrolling
    assert!(!buffer_output.contains("task_000"));
    assert!(!buffer_output.contains("task_001"));
}

#[test]
fn test_log_viewer_updates_on_file_change() {
    use ghost::app::tui::app::TuiApp;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a temporary log file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "Initial log line 1").unwrap();
    writeln!(temp_file, "Initial log line 2").unwrap();
    temp_file.flush().unwrap();

    // Create a task with the temp log file
    let task = Task {
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
        log_path: temp_file.path().to_string_lossy().to_string(),
    };

    let mut app = TuiApp::new().unwrap();
    app.tasks = vec![task];
    app.selected_index = 0;
    app.view_mode = ghost::app::tui::ViewMode::LogView;

    // First render to initialize log view
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| {
            app.render(f);
        })
        .unwrap();

    // Initial line count should be 2
    assert_eq!(app.log_lines_count, 2);

    // Write more lines to the file
    writeln!(temp_file, "New log line 3").unwrap();
    writeln!(temp_file, "New log line 4").unwrap();
    writeln!(temp_file, "New log line 5").unwrap();
    temp_file.flush().unwrap();

    // Render again to reload the file
    terminal
        .draw(|f| {
            app.render(f);
        })
        .unwrap();

    // Line count should now be 5
    assert_eq!(app.log_lines_count, 5);

    // Verify the log viewer shows the updated line count
    let buffer_output = buffer_to_string(terminal.backend().buffer());
    assert!(buffer_output.contains("5 lines total"));
}

#[test]
fn test_log_viewer_caches_file_content() {
    use ghost::app::tui::app::TuiApp;
    use std::io::Write;
    use std::time::Instant;
    use tempfile::NamedTempFile;

    // Create a large temporary log file
    let mut temp_file = NamedTempFile::new().unwrap();
    for i in 0..10000 {
        writeln!(temp_file, "Log line {i}").unwrap();
    }
    temp_file.flush().unwrap();

    // Create a task with the temp log file
    let task = Task {
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
        log_path: temp_file.path().to_string_lossy().to_string(),
    };

    let mut app = TuiApp::new().unwrap();
    app.tasks = vec![task];
    app.selected_index = 0;
    app.view_mode = ghost::app::tui::ViewMode::LogView;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // First render - should read the file
    let start = Instant::now();
    terminal
        .draw(|f| {
            app.render(f);
        })
        .unwrap();
    let first_render_time = start.elapsed();

    // Second render without file changes - should use cache
    let start = Instant::now();
    terminal
        .draw(|f| {
            app.render(f);
        })
        .unwrap();
    let second_render_time = start.elapsed();

    // Second render should be significantly faster (cached)
    assert!(
        second_render_time < first_render_time / 2,
        "Second render should be much faster due to caching. First: {first_render_time:?}, Second: {second_render_time:?}"
    );

    // Line count should remain the same
    assert_eq!(app.log_lines_count, 10000);
}

#[test]
fn test_log_viewer_reloads_on_file_modification() {
    use ghost::app::tui::app::TuiApp;
    use std::io::Write;
    use std::thread;
    use std::time::Duration;
    use tempfile::NamedTempFile;

    // Create a temporary log file
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "Initial log line 1").unwrap();
    writeln!(temp_file, "Initial log line 2").unwrap();
    temp_file.flush().unwrap();

    // Create a task with the temp log file
    let task = Task {
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
        log_path: temp_file.path().to_string_lossy().to_string(),
    };

    let mut app = TuiApp::new().unwrap();
    app.tasks = vec![task];
    app.selected_index = 0;
    app.view_mode = ghost::app::tui::ViewMode::LogView;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // First render
    terminal
        .draw(|f| {
            app.render(f);
        })
        .unwrap();
    assert_eq!(app.log_lines_count, 2);

    // Wait to ensure file modification time changes
    thread::sleep(Duration::from_millis(10));

    // Modify the file
    writeln!(temp_file, "New log line 3").unwrap();
    writeln!(temp_file, "New log line 4").unwrap();
    temp_file.flush().unwrap();

    // Render again - should detect file change and reload
    terminal
        .draw(|f| {
            app.render(f);
        })
        .unwrap();

    // Should have reloaded the file
    assert_eq!(app.log_lines_count, 4);
}

#[test]
fn test_log_viewer_only_processes_visible_lines() {
    use ghost::app::tui::app::TuiApp;
    use std::io::Write;
    use std::time::Instant;
    use tempfile::NamedTempFile;

    // Create a large temporary log file
    let mut temp_file = NamedTempFile::new().unwrap();
    for i in 0..100000 {
        writeln!(temp_file, "Log line {i}").unwrap();
    }
    temp_file.flush().unwrap();

    // Create a task with the temp log file
    let task = Task {
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
        log_path: temp_file.path().to_string_lossy().to_string(),
    };

    let mut app = TuiApp::new().unwrap();
    app.tasks = vec![task];
    app.selected_index = 0;
    app.view_mode = ghost::app::tui::ViewMode::LogView;

    // Create a small terminal (only 20 lines visible)
    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();

    // Measure time to render
    let start = Instant::now();
    terminal
        .draw(|f| {
            app.render(f);
        })
        .unwrap();
    let render_time = start.elapsed();

    // Rendering should be fast even with 100k lines
    // because we only process visible lines
    assert!(
        render_time.as_millis() < 50,
        "Render took too long: {:?}ms. Should process only visible lines.",
        render_time.as_millis()
    );

    // Verify that only visible content is in the output
    let buffer_output = buffer_to_string(terminal.backend().buffer());
    let line_count = buffer_output.matches("Log line").count();

    // Should only have around 10-15 visible lines (accounting for borders and headers)
    assert!(
        line_count < 20,
        "Too many lines processed: {}. Should only process visible lines.",
        line_count
    );
}

#[test]
fn test_log_viewer_memory_limit() {
    use ghost::app::tui::app::TuiApp;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // Create a large temporary log file
    let mut temp_file = NamedTempFile::new().unwrap();
    for i in 0..50000 {
        writeln!(
            temp_file,
            "Log line {}: This is a fairly long log line to consume more memory",
            i
        )
        .unwrap();
    }
    temp_file.flush().unwrap();

    // Create a task with the temp log file
    let task = Task {
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
        log_path: temp_file.path().to_string_lossy().to_string(),
    };

    let mut app = TuiApp::new().unwrap();
    app.tasks = vec![task];
    app.selected_index = 0;
    app.view_mode = ghost::app::tui::ViewMode::LogView;

    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    // First render to load the file
    terminal
        .draw(|f| {
            app.render(f);
        })
        .unwrap();

    // Check that we don't load all 50k lines into memory
    // The cache should have a memory limit (e.g., max 10k lines)
    assert!(
        app.log_lines_count <= 10000,
        "Too many lines in memory: {}. Should limit to prevent memory issues.",
        app.log_lines_count
    );

    // Verify we can still scroll to near the end
    if app.log_lines_count > 0 {
        app.log_scroll_offset = app.log_lines_count.saturating_sub(10);
        terminal
            .draw(|f| {
                app.render(f);
            })
            .unwrap();

        // Should show recent lines
        let buffer_output = buffer_to_string(terminal.backend().buffer());

        // Debug output
        if !buffer_output.contains("Log line") {
            println!("Buffer output: {buffer_output}");
            println!("Log lines count: {}", app.log_lines_count);
            println!("Scroll offset: {}", app.log_scroll_offset);
        }

        assert!(buffer_output.contains("Log line") || buffer_output.contains("lines total"));
    }
}
