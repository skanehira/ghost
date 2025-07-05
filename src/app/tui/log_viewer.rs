use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use std::fs;
use std::io::{BufRead, BufReader};

use crate::app::storage::task::Task;

/// Maximum number of lines to keep in memory
const MAX_LINES_IN_MEMORY: usize = 10_000;

pub struct LogViewerWidget<'a> {
    task: &'a Task,
    log_lines: Vec<String>,
    scroll_offset: usize,
}

impl<'a> LogViewerWidget<'a> {
    pub fn new(task: &'a Task) -> Self {
        let mut viewer = Self {
            task,
            log_lines: Vec::new(),
            scroll_offset: 0,
        };
        viewer.load_log_content();
        viewer
    }

    pub fn with_scroll_offset(task: &'a Task, scroll_offset: usize) -> Self {
        let mut viewer = Self {
            task,
            log_lines: Vec::new(),
            scroll_offset,
        };
        viewer.load_log_content();
        viewer
    }

    pub fn with_cached_content(
        task: &'a Task,
        scroll_offset: usize,
        log_lines: Vec<String>,
    ) -> Self {
        Self {
            task,
            log_lines,
            scroll_offset,
        }
    }

    fn load_log_content(&mut self) {
        use std::collections::VecDeque;

        match fs::File::open(&self.task.log_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut lines = VecDeque::new();

                // Read lines with memory limit
                for line_result in reader.lines() {
                    match line_result {
                        Ok(line) => {
                            lines.push_back(line);
                            // Keep only the most recent lines
                            if lines.len() > MAX_LINES_IN_MEMORY {
                                lines.pop_front();
                            }
                        }
                        Err(_) => {
                            lines.push_back("Error reading line".to_string());
                        }
                    }
                }

                self.log_lines = lines.into_iter().collect();
            }
            Err(_) => {
                self.log_lines = vec!["Log file not found or cannot be read".to_string()];
            }
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset < self.log_lines.len().saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scroll_to_bottom(&mut self) {
        if !self.log_lines.is_empty() {
            self.scroll_offset = self.log_lines.len().saturating_sub(1);
        }
    }

    pub fn get_lines_count(&self) -> usize {
        self.log_lines.len()
    }

    pub fn get_lines(&self) -> &[String] {
        &self.log_lines
    }

    fn get_styled_lines(&self, viewport_height: u16) -> Vec<Line<'static>> {
        if self.log_lines.is_empty() {
            return vec![Line::from("No log content available")];
        }

        // Calculate visible range based on scroll position
        let content_height = viewport_height.saturating_sub(2) as usize; // Account for borders
        let scroll_position = self.calculate_scroll_offset(viewport_height) as usize;

        let start_idx = scroll_position;
        let end_idx = (start_idx + content_height).min(self.log_lines.len());

        // Only process visible lines
        self.log_lines[start_idx..end_idx]
            .iter()
            .enumerate()
            .map(|(relative_idx, line)| {
                let absolute_idx = start_idx + relative_idx;
                let content = if line.len() > 100 {
                    let truncated = &line[..97];
                    format!("{truncated}...")
                } else {
                    line.to_string()
                };

                // Highlight the current line
                if absolute_idx == self.scroll_offset {
                    Line::from(vec![Span::styled(
                        content,
                        Style::default().bg(Color::DarkGray),
                    )])
                } else {
                    Line::from(vec![Span::raw(content)])
                }
            })
            .collect()
    }

    fn calculate_scroll_offset(&self, viewport_height: u16) -> u16 {
        // Calculate proper scroll offset to keep the current line visible
        let content_height = viewport_height.saturating_sub(2); // Account for borders

        if self.scroll_offset == 0 {
            return 0;
        }

        // If we're near the bottom, adjust scroll to show the current line
        let max_visible_offset = self
            .scroll_offset
            .saturating_sub(content_height as usize / 2);
        max_visible_offset as u16
    }
}

impl<'a> Widget for LogViewerWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Header
        let header_text = format!(
            " Log View: {} ({}) ",
            &self.task.id[0..8],
            self.task.command.split('"').nth(1).unwrap_or("unknown")
        );
        let header = Paragraph::new(header_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Log Viewer ")
                    .title_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default());
        header.render(layout[0], buf);

        // Content
        let styled_lines = self.get_styled_lines(layout[1].height);
        let scroll_offset = self.calculate_scroll_offset(layout[1].height);
        let log_paragraph = Paragraph::new(styled_lines)
            .block(Block::default().borders(Borders::ALL).title({
                let lines_len = self.log_lines.len();
                format!(" {lines_len} lines total ")
            }))
            .style(Style::default())
            .wrap(Wrap { trim: false })
            .scroll((scroll_offset, 0));
        log_paragraph.render(layout[1], buf);

        // Footer with key bindings
        let footer_text = " j/k:Scroll  gg/G:Top/Bottom  Esc:Back ";
        let footer = Paragraph::new(footer_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default());
        footer.render(layout[2], buf);
    }
}

pub struct LogViewer {
    pub current_task: Option<Task>,
    pub viewer_widget: Option<LogViewerWidget<'static>>,
}

impl Default for LogViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl LogViewer {
    pub fn new() -> Self {
        Self {
            current_task: None,
            viewer_widget: None,
        }
    }

    pub fn set_task(&mut self, task: Task) {
        self.current_task = Some(task);
        // Note: In real implementation, we'd need to handle the lifetime properly
        // For now, we'll create the widget in the render method
    }

    pub fn scroll_up(&mut self) {
        // Implementation would interact with the widget
    }

    pub fn scroll_down(&mut self) {
        // Implementation would interact with the widget
    }

    pub fn scroll_to_top(&mut self) {
        // Implementation would interact with the widget
    }

    pub fn scroll_to_bottom(&mut self) {
        // Implementation would interact with the widget
    }
}
