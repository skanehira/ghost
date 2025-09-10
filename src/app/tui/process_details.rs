use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect, Size},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use tui_scrollview::{ScrollView, ScrollViewState, ScrollbarVisibility};

use crate::app::port_detector::detect_listening_ports;
use crate::app::storage::task::Task;
use crate::app::storage::task_status::TaskStatus;
use chrono::{TimeZone, Utc};

pub struct ProcessDetailsWidget<'a> {
    task: &'a Task,
}

impl<'a> ProcessDetailsWidget<'a> {
    pub fn new(task: &'a Task) -> Self {
        Self { task }
    }

    fn format_command(&self) -> String {
        // Parse JSON command
        if let Ok(command_vec) = serde_json::from_str::<Vec<String>>(&self.task.command) {
            command_vec.join(" ")
        } else {
            // Fallback if parsing fails
            self.task.command.clone()
        }
    }

    pub fn render(self, frame: &mut Frame, area: Rect, env_scroll_state: &mut ScrollViewState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8), // Basic info section (6 lines + 2 borders)
                Constraint::Length(5), // Listening ports section
                Constraint::Min(5),    // Environment variables section
                Constraint::Length(2), // Footer
            ])
            .split(area);

        // Render basic info section
        self.render_basic_info(frame, chunks[0]);

        // Render listening ports section
        self.render_listening_ports(frame, chunks[1]);

        // Render environment variables section
        self.render_environment_variables(frame, chunks[2], env_scroll_state);

        // Render footer
        self.render_footer(frame, chunks[3]);
    }

    fn render_basic_info(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Process Details ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        // Calculate runtime
        let runtime = {
            let started = Utc.timestamp_opt(self.task.started_at, 0).single().unwrap();
            let elapsed = if self.task.status == TaskStatus::Running {
                let now = Utc::now();
                now.signed_duration_since(started)
            } else if let Some(finished_at) = self.task.finished_at {
                let ended = Utc.timestamp_opt(finished_at, 0).single().unwrap();
                ended.signed_duration_since(started)
            } else {
                chrono::Duration::zero()
            };

            let hours = elapsed.num_hours();
            let minutes = elapsed.num_minutes() % 60;
            let seconds = elapsed.num_seconds() % 60;

            if hours > 0 {
                format!("{hours}h {minutes}m {seconds}s")
            } else if minutes > 0 {
                format!("{minutes}m {seconds}s")
            } else {
                format!("{seconds}s")
            }
        };

        // Format status with color
        let status_style = match self.task.status {
            TaskStatus::Running => Style::default().fg(Color::Green),
            TaskStatus::Exited => Style::default().fg(Color::Yellow),
            TaskStatus::Killed => Style::default().fg(Color::Red),
            TaskStatus::Unknown => Style::default().fg(Color::Gray),
        };

        let status_text = format!("{} ({})", self.task.status.as_str(), runtime);

        // Build info lines
        let info_lines = vec![
            Line::from(vec![
                Span::styled("Task ID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&self.task.id),
            ]),
            Line::from(vec![
                Span::styled("Command: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(self.format_command()),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(status_text, status_style),
            ]),
            Line::from(vec![
                Span::styled("PID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(self.task.pid.to_string()),
                Span::raw(" | "),
                Span::styled("PGID: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(self.task.pgid.map_or("N/A".to_string(), |p| p.to_string())),
            ]),
            Line::from(vec![
                Span::styled("Directory: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(self.task.cwd.as_deref().unwrap_or("N/A")),
            ]),
            Line::from(vec![
                Span::styled("Log File: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&self.task.log_path),
            ]),
        ];

        let paragraph = Paragraph::new(info_lines)
            .block(block)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn render_listening_ports(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Listening Ports ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let port_lines = if self.task.status == TaskStatus::Running {
            // Get actual listening ports for running processes
            match detect_listening_ports(self.task.pid) {
                Ok(ports) => {
                    if ports.is_empty() {
                        vec![Line::from(Span::styled(
                            "Not listening on any ports",
                            Style::default().fg(Color::DarkGray),
                        ))]
                    } else {
                        ports
                            .into_iter()
                            .map(|port| {
                                Line::from(vec![
                                    Span::styled(
                                        format!("{:<6}", port.protocol),
                                        Style::default().fg(Color::Blue),
                                    ),
                                    Span::raw(format!("{:<20}", port.local_addr)),
                                    Span::styled(
                                        port.state.clone(),
                                        Style::default().fg(Color::Green),
                                    ),
                                ])
                            })
                            .collect()
                    }
                }
                Err(e) => {
                    // Check if it's a command not found error
                    if let crate::app::error::GhostError::CommandNotFound { command } = e {
                        vec![
                            Line::from(Span::styled(
                                format!("{command} command not found"),
                                Style::default().fg(Color::Yellow),
                            )),
                            Line::from(Span::styled(
                                format!("Please install {command} to enable port detection"),
                                Style::default().fg(Color::DarkGray),
                            )),
                        ]
                    } else {
                        vec![Line::from(Span::styled(
                            format!("Failed to detect ports: {e:?}"),
                            Style::default().fg(Color::Red),
                        ))]
                    }
                }
            }
        } else {
            vec![Line::from(Span::styled(
                "Process not running",
                Style::default().fg(Color::DarkGray),
            ))]
        };

        let paragraph = Paragraph::new(port_lines)
            .block(block)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn render_environment_variables(
        &self,
        frame: &mut Frame,
        area: Rect,
        scroll_state: &mut ScrollViewState,
    ) {
        // Split the area into content and footer separator
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        let block = Block::default()
            .title(" Environment Variables ")
            .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(chunks[0]);

        // Prepare environment variables content
        let env_lines: Vec<String> = if let Some(env_json) = &self.task.env {
            // Parse JSON environment variables
            if let Ok(env_map) = serde_json::from_str::<Vec<(String, String)>>(env_json) {
                env_map
                    .iter()
                    .map(|(key, value)| format!("{key}={value}"))
                    .collect()
            } else {
                vec!["Failed to parse environment variables".to_string()]
            }
        } else {
            vec!["No environment variables set".to_string()]
        };

        // Render the block first
        frame.render_widget(block, chunks[0]);

        // Calculate content size - use inner width to avoid horizontal scrolling
        let content_height = env_lines.len() as u16;
        let content_size = Size::new(inner.width, content_height);

        // Create scrollview widget with proper size and hide horizontal scrollbar
        let mut scroll_view = ScrollView::new(content_size)
            .horizontal_scrollbar_visibility(ScrollbarVisibility::Never)
            .vertical_scrollbar_visibility(ScrollbarVisibility::Never);

        // Render environment variables with wrapping
        let env_text = env_lines.join("\n");
        let env_paragraph = Paragraph::new(env_text)
            .style(Style::default())
            .wrap(Wrap { trim: false });

        // Use the inner width for rendering to enable text wrapping
        scroll_view.render_widget(env_paragraph, Rect::new(0, 0, inner.width, content_height));

        // Render the scrollview
        frame.render_stateful_widget(scroll_view, inner, scroll_state);

        // Draw the separator line between environment variables and footer
        // Using direct buffer manipulation like LogViewerScrollWidget
        let buf = frame.buffer_mut();
        if chunks[1].y > 0 && chunks[0].width > 0 {
            // Left connection: ├
            buf[(chunks[0].x, chunks[1].y)]
                .set_symbol(symbols::line::VERTICAL_RIGHT)
                .set_style(Style::default().fg(Color::Cyan));

            // Horizontal line
            for x in chunks[0].x + 1..chunks[0].x + chunks[0].width - 1 {
                buf[(x, chunks[1].y)]
                    .set_symbol(symbols::line::HORIZONTAL)
                    .set_style(Style::default().fg(Color::Cyan));
            }

            // Right connection: ┤
            buf[(chunks[0].x + chunks[0].width - 1, chunks[1].y)]
                .set_symbol(symbols::line::VERTICAL_LEFT)
                .set_style(Style::default().fg(Color::Cyan));
        }
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        // Render keybinds
        let keybinds = vec![
            Span::styled("[q]", Style::default().fg(Color::Yellow)),
            Span::raw(" Quit  "),
            Span::styled("[Esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" Back to list  "),
            Span::styled("[j/k]", Style::default().fg(Color::Yellow)),
            Span::raw(" Scroll  "),
            Span::styled("[C-d/C-u]", Style::default().fg(Color::Yellow)),
            Span::raw(" Page"),
        ];

        let keybind_paragraph = Paragraph::new(Line::from(keybinds))
            .style(Style::default())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
                    .border_style(Style::default().fg(Color::Cyan)),
            );

        frame.render_widget(keybind_paragraph, area);
    }
}
