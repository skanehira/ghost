use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use std::fs;
use std::io::{BufRead, BufReader};

use crate::app::storage::task::Task;

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

    fn load_log_content(&mut self) {
        match fs::File::open(&self.task.log_path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                self.log_lines = reader
                    .lines()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap_or_else(|_| vec!["Error reading log file".to_string()]);
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

    fn get_styled_lines(&self) -> Vec<Line<'static>> {
        if self.log_lines.is_empty() {
            return vec![Line::from("No log content available")];
        }

        self.log_lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let content = if line.len() > 100 {
                    format!("{}...", &line[..97])
                } else {
                    line.to_string()
                };

                // Highlight the current line
                if i == self.scroll_offset {
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
        let styled_lines = self.get_styled_lines();
        let scroll_offset = self.calculate_scroll_offset(layout[1].height);
        let log_paragraph = Paragraph::new(styled_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} lines total ", self.log_lines.len())),
            )
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
