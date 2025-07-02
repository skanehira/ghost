use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
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
        let task_list_widget = TaskListWidget::new(&self.tasks, &self.filter, self.selected_index)
            .with_scroll_offset(self.table_scroll_offset);
        frame.render_widget(task_list_widget, area);
    }
}

pub struct TaskListWidget<'a> {
    tasks: &'a [Task],
    filter: &'a TaskFilter,
    selected_index: usize,
    scroll_offset: usize,
}

impl<'a> TaskListWidget<'a> {
    pub fn new(tasks: &'a [Task], filter: &'a TaskFilter, selected_index: usize) -> Self {
        Self {
            tasks,
            filter,
            selected_index,
            scroll_offset: 0,
        }
    }

    pub fn with_scroll_offset(mut self, scroll_offset: usize) -> Self {
        self.scroll_offset = scroll_offset;
        self
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
        // Split the area into content and footer
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Render content (table)
        self.render_content(layout[0], buf);

        // Render footer
        self.render_footer(layout[1], buf);
    }
}

impl<'a> TaskListWidget<'a> {
    fn render_content(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let title = format!(" Ghost TUI v0.0.1 [Filter: {}] ", self.filter_name());

        if self.tasks.is_empty() {
            // Empty state
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
                .block(Block::default().borders(Borders::ALL).title(title));

            table.render(area, buf);
        } else {
            // Table with tasks - apply scrolling
            let visible_tasks: Vec<&Task> = self.tasks.iter().skip(self.scroll_offset).collect();

            let rows: Vec<Row> = visible_tasks
                .iter()
                .enumerate()
                .map(|(display_index, task)| {
                    let actual_index = self.scroll_offset + display_index;
                    let style = if actual_index == self.selected_index {
                        Style::default().bg(Color::LightGreen).fg(Color::Black)
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
                .block(Block::default().borders(Borders::ALL).title(title));

            table.render(area, buf);
        }
    }

    fn render_footer(&self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let keybinds_text = " j/k:Move  l:Log  q:Quit  g/G:Top/Bottom ";
        let footer = Paragraph::new(keybinds_text)
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default());
        footer.render(area, buf);
    }
}
