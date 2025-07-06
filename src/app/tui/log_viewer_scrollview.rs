use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect, Size},
    style::{Color, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};
use serde_json;
use tui_scrollview::{ScrollView, ScrollViewState};

use crate::app::storage::task::Task;

const MAX_LINES_IN_MEMORY: usize = 10_000;

/// A log viewer widget using tui-scrollview for efficient scrolling
pub struct LogViewerScrollWidget {
    lines: Vec<String>,
    task_id: String,
    command: String,
}

impl LogViewerScrollWidget {
    /// Create a new log viewer with loaded content
    pub fn new(task: &Task) -> Self {
        let lines = Self::load_log_file(&task.log_path);
        Self {
            lines,
            task_id: task.id[0..8].to_string(),
            command: Self::parse_command(&task.command),
        }
    }

    /// Create with cached content
    pub fn with_cached_content(task: &Task, cached_lines: Vec<String>) -> Self {
        Self {
            lines: cached_lines,
            task_id: task.id[0..8].to_string(),
            command: Self::parse_command(&task.command),
        }
    }

    /// Load incremental content from a file
    pub fn load_incremental_content(
        task: &Task,
        mut existing_lines: Vec<String>,
        previous_size: u64,
    ) -> Self {
        // Try to read only the new content
        if let Ok(mut file) = std::fs::File::open(&task.log_path) {
            use std::io::{Read, Seek, SeekFrom};

            // Seek to the previous end position
            if file.seek(SeekFrom::Start(previous_size)).is_ok() {
                let mut new_content = String::new();
                if file.read_to_string(&mut new_content).is_ok() {
                    // Append new lines to existing lines
                    for line in new_content.lines() {
                        existing_lines.push(line.to_string());
                    }

                    // Apply memory limit if needed
                    if existing_lines.len() > MAX_LINES_IN_MEMORY {
                        let skip_count = existing_lines.len() - MAX_LINES_IN_MEMORY;
                        existing_lines = existing_lines.into_iter().skip(skip_count).collect();
                    }
                }
            }
        }

        Self {
            lines: existing_lines,
            task_id: task.id[0..8].to_string(),
            command: Self::parse_command(&task.command),
        }
    }

    /// Load log file with memory limit
    fn load_log_file(path: &str) -> Vec<String> {
        match std::fs::read_to_string(path) {
            Ok(content) => {
                let lines: Vec<String> = content.lines().map(String::from).collect();
                if lines.len() > MAX_LINES_IN_MEMORY {
                    // Take last MAX_LINES_IN_MEMORY lines
                    let skip_count = lines.len() - MAX_LINES_IN_MEMORY;
                    lines.into_iter().skip(skip_count).collect()
                } else {
                    lines
                }
            }
            Err(_) => vec!["Error: Could not read log file".to_string()],
        }
    }

    /// Get the lines for external use (caching)
    pub fn get_lines(&self) -> &[String] {
        &self.lines
    }

    /// Get the total line count
    pub fn get_lines_count(&self) -> usize {
        self.lines.len()
    }

    /// Parse command from JSON format to readable string
    fn parse_command(command_json: &str) -> String {
        // Try to parse the JSON array format like ["npm","run","dev"]
        if let Ok(parsed) = serde_json::from_str::<Vec<String>>(command_json) {
            parsed.join(" ")
        } else {
            // If parsing fails, return the original string
            command_json.to_string()
        }
    }

    /// Create footer widget
    fn create_footer(&self) -> Paragraph {
        let keybinds = " j/k:Scroll  h/l:Horizontal  gg/G:Top/Bottom  Esc:Back  q:Quit ";

        Paragraph::new(keybinds).block(Block::default().borders(Borders::ALL))
    }
}

impl StatefulWidget for LogViewerScrollWidget {
    type State = ScrollViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Layout: content + footer (3)
        let chunks = Layout::vertical([Constraint::Min(5), Constraint::Length(3)]).split(area);

        // Render footer
        self.create_footer().render(chunks[1], buf);

        // Calculate content size (lines count, max line width)
        let content_width = self
            .lines
            .iter()
            .map(|line| line.len() + 7) // +7 for line number
            .max()
            .unwrap_or(80) as u16;

        let content_height = self.lines.len() as u16;
        let content_size = Size::new(content_width, content_height);

        // Create a block for the content area with borders and title
        let title = format!(" {} - {} ", self.task_id, self.command);
        let content_block = Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title(title);

        // Get the inner area for the scroll view
        let content_inner = content_block.inner(chunks[0]);

        // Render the content block
        content_block.render(chunks[0], buf);

        // Draw the separator line between content and footer
        // The separator is at the top of the footer block (chunks[1].y)
        if chunks[1].y > 0 {
            // Left connection: ├
            buf[(chunks[0].x, chunks[1].y)].set_symbol(symbols::line::VERTICAL_RIGHT);

            // Horizontal line
            for x in chunks[0].x + 1..chunks[0].x + chunks[0].width - 1 {
                buf[(x, chunks[1].y)].set_symbol(symbols::line::HORIZONTAL);
            }

            // Right connection: ┤
            buf[(chunks[0].x + chunks[0].width - 1, chunks[1].y)]
                .set_symbol(symbols::line::VERTICAL_LEFT);
        }

        // Create scroll view with content size and hide scrollbars
        let mut scroll_view = ScrollView::new(content_size)
            .scrollbars_visibility(tui_scrollview::ScrollbarVisibility::Never);

        // Create line numbers paragraph
        let line_numbers: Vec<Line> = self
            .lines
            .iter()
            .enumerate()
            .map(|(idx, _)| {
                let line_number = format!("{:6} ", idx + 1);
                Line::from(Span::styled(
                    line_number,
                    Style::default().fg(Color::DarkGray),
                ))
            })
            .collect();
        let line_numbers_paragraph = Paragraph::new(line_numbers);

        // Create content paragraph
        let content_lines: Vec<Line> = self
            .lines
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect();
        let content_paragraph = Paragraph::new(content_lines);

        // Render line numbers and content inside scroll view
        scroll_view.render_widget(line_numbers_paragraph, Rect::new(0, 0, 7, content_height));
        scroll_view.render_widget(
            content_paragraph,
            Rect::new(7, 0, content_width.saturating_sub(7), content_height),
        );

        // Render the scroll view in the inner content area
        scroll_view.render(content_inner, buf, state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_task(log_path: String) -> Task {
        Task {
            id: "test_task_12345678".to_string(),
            pid: 12345,
            pgid: Some(12345),
            command: r#"["echo","test"]"#.to_string(),
            env: None,
            cwd: None,
            status: crate::app::storage::task_status::TaskStatus::Running,
            exit_code: None,
            started_at: 1704109200,
            finished_at: None,
            log_path,
        }
    }

    #[test]
    fn test_basic_rendering() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Line 1").unwrap();
        writeln!(temp_file, "Line 2").unwrap();
        writeln!(temp_file, "Line 3").unwrap();
        temp_file.flush().unwrap();

        let task = create_test_task(temp_file.path().to_string_lossy().to_string());
        let widget = LogViewerScrollWidget::new(&task);

        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut scroll_state = ScrollViewState::default();

        terminal
            .draw(|f| {
                widget.render(f.area(), f.buffer_mut(), &mut scroll_state);
            })
            .unwrap();

        let buffer = terminal.backend().buffer();
        let content = buffer_to_string(buffer);

        // Check title
        assert!(content.contains("test_tas"));
        assert!(content.contains("echo test"));

        // Check footer
        assert!(content.contains("j/k:Scroll"));
        assert!(content.contains("h/l:Horizontal"));
        assert!(content.contains("gg/G:Top/Bottom"));

        // Check content with line numbers
        assert!(content.contains("     1 Line 1"));
        assert!(content.contains("     2 Line 2"));
        assert!(content.contains("     3 Line 3"));
    }

    #[test]
    fn test_memory_limit() {
        let mut temp_file = NamedTempFile::new().unwrap();
        // Write more than MAX_LINES_IN_MEMORY lines
        for i in 0..15000 {
            writeln!(temp_file, "Line {}", i).unwrap();
        }
        temp_file.flush().unwrap();

        let task = create_test_task(temp_file.path().to_string_lossy().to_string());
        let widget = LogViewerScrollWidget::new(&task);

        // Should only load MAX_LINES_IN_MEMORY lines
        assert_eq!(widget.get_lines_count(), MAX_LINES_IN_MEMORY);

        // Should contain the last lines
        let lines = widget.get_lines();
        assert!(lines[0].contains("Line 5000")); // 15000 - 10000 = 5000
        assert!(lines[9999].contains("Line 14999"));
    }

    #[test]
    fn test_error_handling() {
        let task = create_test_task("/non/existent/file.log".to_string());
        let widget = LogViewerScrollWidget::new(&task);

        // Should have error message
        assert_eq!(widget.get_lines_count(), 1);
        assert_eq!(widget.get_lines()[0], "Error: Could not read log file");
    }

    #[test]
    fn test_cached_content() {
        let task = create_test_task("/dummy/path.log".to_string());
        let cached_lines = vec![
            "Cached line 1".to_string(),
            "Cached line 2".to_string(),
            "Cached line 3".to_string(),
        ];

        let widget = LogViewerScrollWidget::with_cached_content(&task, cached_lines.clone());

        assert_eq!(widget.get_lines_count(), 3);
        assert_eq!(widget.get_lines(), &cached_lines);
    }

    #[test]
    fn test_incremental_loading() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Initial line 1").unwrap();
        writeln!(temp_file, "Initial line 2").unwrap();
        temp_file.flush().unwrap();

        let initial_size = temp_file.as_file().metadata().unwrap().len();

        let task = create_test_task(temp_file.path().to_string_lossy().to_string());
        let existing_lines = vec!["Initial line 1".to_string(), "Initial line 2".to_string()];

        // Add more lines
        writeln!(temp_file, "New line 3").unwrap();
        writeln!(temp_file, "New line 4").unwrap();
        temp_file.flush().unwrap();

        // Load incrementally
        let widget =
            LogViewerScrollWidget::load_incremental_content(&task, existing_lines, initial_size);

        assert_eq!(widget.get_lines_count(), 4);
        let lines = widget.get_lines();
        assert_eq!(lines[0], "Initial line 1");
        assert_eq!(lines[1], "Initial line 2");
        assert_eq!(lines[2], "New line 3");
        assert_eq!(lines[3], "New line 4");
    }

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
}
