use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect};

use super::TaskFilter;
use crate::app::config::Config;
use crate::app::error::Result;
use crate::app::storage::task::Task;
use crate::app::storage::task_repository;

pub struct TuiApp {
    pub tasks: Vec<Task>,
    pub selected_index: usize,
    pub filter: TaskFilter,
    pub should_quit: bool,
    config: Config,
}

impl TuiApp {
    pub fn new() -> Result<Self> {
        let config = Config::default();

        Ok(Self {
            tasks: Vec::new(),
            selected_index: 0,
            filter: TaskFilter::All,
            should_quit: false,
            config,
        })
    }

    /// Load tasks from database
    pub fn refresh_tasks(&mut self) -> Result<()> {
        let conn = rusqlite::Connection::open(&self.config.db_path)?;

        // Filter status for database query
        let status_filter = match self.filter {
            TaskFilter::All => None,
            TaskFilter::Running => Some("running"),
            TaskFilter::Exited => Some("exited"),
            TaskFilter::Killed => Some("killed"),
        };

        self.tasks = task_repository::get_tasks_with_process_check(&conn, status_filter)?;

        // Adjust selected index if needed
        if self.selected_index >= self.tasks.len() && !self.tasks.is_empty() {
            self.selected_index = self.tasks.len() - 1;
        }

        Ok(())
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Quit
            KeyCode::Char('q') => {
                self.should_quit = true;
            }

            // Navigation (Vim-like)
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_selection_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_selection_up();
            }

            // Go to top/bottom
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::NONE) => {
                // Handle 'gg' for top - this is simplified, real vim would need state tracking
                self.selected_index = 0;
            }
            KeyCode::Char('G') => {
                if !self.tasks.is_empty() {
                    self.selected_index = self.tasks.len() - 1;
                }
            }

            // Ctrl+C as alternative quit
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            _ => {}
        }

        Ok(())
    }

    fn move_selection_down(&mut self) {
        if !self.tasks.is_empty() && self.selected_index < self.tasks.len() - 1 {
            self.selected_index += 1;
        }
    }

    fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Render the TUI
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        self.render_task_list(frame, area);
    }

    /// Render task list widget
    fn render_task_list(&self, frame: &mut Frame, area: Rect) {
        use super::task_list::TaskListWidget;

        let widget = TaskListWidget::new(&self.tasks, &self.filter, self.selected_index);
        frame.render_widget(widget, area);
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}
