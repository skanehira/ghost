use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Widget},
};

// Layout constants
const ID_COLUMN_WIDTH: u16 = 10;
const PID_COLUMN_WIDTH: u16 = 8;
const STATUS_COLUMN_WIDTH: u16 = 9;
const STARTED_COLUMN_WIDTH: u16 = 19;
const COMMAND_COLUMN_MIN_WIDTH: u16 = 26;

// Column constraints for the table
const COLUMN_CONSTRAINTS: [Constraint; 5] = [
    Constraint::Length(ID_COLUMN_WIDTH),
    Constraint::Length(PID_COLUMN_WIDTH),
    Constraint::Length(STATUS_COLUMN_WIDTH),
    Constraint::Length(STARTED_COLUMN_WIDTH),
    Constraint::Min(COMMAND_COLUMN_MIN_WIDTH),
];

use super::{App, TaskFilter};
use crate::app::storage::task::Task;
use crate::app::storage::task_status::TaskStatus;

impl App {
    pub fn render_task_list(&self, frame: &mut Frame, area: Rect) {
        let task_list_widget = TaskListWidget::new(&self.tasks, &self.filter, self.selected_index);
        frame.render_widget(task_list_widget, area);
    }
}

pub struct TaskListWidget<'a> {
    tasks: &'a [Task],
    filter: &'a TaskFilter,
    selected_index: usize,
}

impl<'a> TaskListWidget<'a> {
    pub fn new(tasks: &'a [Task], filter: &'a TaskFilter, selected_index: usize) -> Self {
        Self {
            tasks,
            filter,
            selected_index,
        }
    }

    fn filter_name(&self) -> &'static str {
        match self.filter {
            TaskFilter::All => "All",
            TaskFilter::Running => "Running",
            TaskFilter::Exited => "Exited",
            TaskFilter::Killed => "Killed",
        }
    }

    fn status_style(&self, status: &TaskStatus) -> Style {
        match status {
            TaskStatus::Running => Style::default().fg(Color::Green),
            TaskStatus::Exited => Style::default().fg(Color::Blue),
            TaskStatus::Killed => Style::default().fg(Color::Red),
            TaskStatus::Unknown => Style::default().fg(Color::Gray),
        }
    }

    fn parse_command(&self, command_json: &str) -> String {
        match serde_json::from_str::<Vec<String>>(command_json) {
            Ok(cmd_vec) => cmd_vec.join(" "),
            Err(_) => command_json.to_string(),
        }
    }

    fn format_timestamp(&self, timestamp: i64) -> String {
        use chrono::{DateTime, Utc};
        let dt = DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap();
        dt.format("%Y-%m-%d %H:%M").to_string()
    }

    fn create_header_row(&self) -> Row {
        Row::new(vec![
            Cell::from(" ID"),
            Cell::from(" PID"),
            Cell::from(" Status"),
            Cell::from(" Started"),
            Cell::from(" Command"),
        ])
        .style(Style::default())
    }
}

impl<'a> Widget for TaskListWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let title = format!(" Ghost TUI v0.0.1 [Filter: {}] ", self.filter_name());

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default());

        if self.tasks.is_empty() {
            // Empty state with footer
            let rows = vec![
                Row::new(vec![
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ]),
                Row::new(vec![
                    Cell::from(""),
                    Cell::from("No tasks"),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ]),
                Row::new(vec![
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                    Cell::from(""),
                ]),
            ];

            let table = Table::new(rows, COLUMN_CONSTRAINTS)
                .header(self.create_header_row())
                .block(block);

            table.render(area, buf);

            // Render footer separator manually
            self.render_footer_separator(area, buf);
            self.render_footer_text(area, buf);
            self.render_bottom_border(area, buf);
        } else {
            // Table with tasks
            let rows: Vec<Row> = self
                .tasks
                .iter()
                .enumerate()
                .map(|(i, task)| {
                    let style = if i == self.selected_index {
                        Style::default().bg(Color::Yellow).fg(Color::Black)
                    } else {
                        Style::default()
                    };

                    let status_style = self.status_style(&task.status);

                    Row::new(vec![
                        Cell::from(format!(" {}", &task.id[0..8])), // Keep short ID
                        Cell::from(format!(" {}", task.pid)),
                        Cell::from(format!(" {}", task.status.as_str())).style(status_style),
                        Cell::from(format!(" {}", self.format_timestamp(task.started_at))),
                        Cell::from(format!(" {}", self.parse_command(&task.command))),
                    ])
                    .style(style)
                })
                .collect();

            let table = Table::new(rows, COLUMN_CONSTRAINTS)
                .header(self.create_header_row())
                .block(block);

            table.render(area, buf);

            // Render footer separator manually
            self.render_footer_separator(area, buf);
            self.render_footer_text(area, buf);
            self.render_bottom_border(area, buf);
        }
    }
}

impl<'a> TaskListWidget<'a> {
    fn render_footer_separator(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let y = area.y + area.height - 3; // Third to last row (above footer text)
        let x_start = area.x;
        let x_end = area.x + area.width - 1;

        // Draw the separator line
        buf[(x_start, y)].set_symbol("├");
        for x in x_start + 1..x_end {
            buf[(x, y)].set_symbol("─");
        }
        buf[(x_end, y)].set_symbol("┤");
    }

    fn render_footer_text(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let y = area.y + area.height - 2; // Second to last row
        let text = " j/k:Move  l:Log  q:Quit  g/G:Top/Bottom";
        let x_start = area.x + 1; // Inside the border

        // Draw border characters
        buf[(area.x, y)].set_symbol("│");
        buf[(area.x + area.width - 1, y)].set_symbol("│");

        // Draw the text
        for (i, ch) in text.chars().enumerate() {
            let x = x_start + i as u16;
            if x < area.x + area.width - 1 {
                buf[(x, y)].set_symbol(&ch.to_string());
            }
        }

        // Fill remaining space with spaces inside the border
        for x in (x_start + text.len() as u16)..(area.x + area.width - 1) {
            buf[(x, y)].set_symbol(" ");
        }
    }

    fn render_bottom_border(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let y = area.y + area.height - 1; // Last row
        let x_start = area.x;
        let x_end = area.x + area.width - 1;

        // Draw the bottom border
        buf[(x_start, y)].set_symbol("└");
        for x in x_start + 1..x_end {
            buf[(x, y)].set_symbol("─");
        }
        buf[(x_end, y)].set_symbol("┘");
    }
}

/// Footer widget displaying key bindings
pub struct FooterWidget;

impl Default for FooterWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl FooterWidget {
    pub fn new() -> Self {
        Self
    }
}

impl Widget for FooterWidget {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let keybinds_text = " j/k:Move  q:Quit  g/G:Top/Bottom";

        let footer = Paragraph::new(keybinds_text).style(Style::default()).block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default()),
        );

        footer.render(area, buf);
    }
}
