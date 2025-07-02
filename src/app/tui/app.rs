use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect};

use super::log_viewer::LogViewerWidget;
use super::{TaskFilter, ViewMode};
use crate::app::config::Config;
use crate::app::error::Result;
use crate::app::storage::task::Task;
use crate::app::storage::task_repository;

pub struct TuiApp {
    pub tasks: Vec<Task>,
    pub selected_index: usize,
    pub filter: TaskFilter,
    pub should_quit: bool,
    pub view_mode: ViewMode,
    pub log_scroll_offset: usize,
    pub log_lines_count: usize,
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
            view_mode: ViewMode::TaskList,
            log_scroll_offset: 0,
            log_lines_count: 0,
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
        match self.view_mode {
            ViewMode::TaskList => self.handle_task_list_key(key),
            ViewMode::LogView => self.handle_log_view_key(key),
        }
    }

    fn handle_task_list_key(&mut self, key: KeyEvent) -> Result<()> {
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

            // Log view
            KeyCode::Char('l') => {
                if !self.tasks.is_empty() {
                    self.view_mode = ViewMode::LogView;
                    self.initialize_log_view();
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

    fn handle_log_view_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Return to task list
            KeyCode::Esc => {
                self.view_mode = ViewMode::TaskList;
            }

            // Quit
            KeyCode::Char('q') => {
                self.should_quit = true;
            }

            // Scroll in log view
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_log_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_log_up();
            }

            // Go to top/bottom in log
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::NONE) => {
                self.log_scroll_offset = 0;
            }
            KeyCode::Char('G') => {
                if self.log_lines_count > 0 {
                    self.log_scroll_offset = self.log_lines_count.saturating_sub(1);
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

    fn initialize_log_view(&mut self) {
        use std::fs;
        use std::io::{BufRead, BufReader};

        if !self.tasks.is_empty() && self.selected_index < self.tasks.len() {
            let selected_task = &self.tasks[self.selected_index];

            // Count lines in log file
            match fs::File::open(&selected_task.log_path) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    self.log_lines_count = reader.lines().count();
                    // Start at the bottom of the log (most recent content)
                    if self.log_lines_count > 0 {
                        self.log_scroll_offset = self.log_lines_count.saturating_sub(1);
                    } else {
                        self.log_scroll_offset = 0;
                    }
                }
                Err(_) => {
                    self.log_lines_count = 1; // Error message
                    self.log_scroll_offset = 0;
                }
            }
        }
    }

    fn scroll_log_down(&mut self) {
        if self.log_scroll_offset < self.log_lines_count.saturating_sub(1) {
            self.log_scroll_offset += 1;
        }
    }

    fn scroll_log_up(&mut self) {
        if self.log_scroll_offset > 0 {
            self.log_scroll_offset -= 1;
        }
    }

    /// Render the TUI
    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        match self.view_mode {
            ViewMode::TaskList => self.render_task_list(frame, area),
            ViewMode::LogView => self.render_log_view(frame, area),
        }
    }

    /// Render task list widget
    fn render_task_list(&self, frame: &mut Frame, area: Rect) {
        use super::task_list::TaskListWidget;

        let widget = TaskListWidget::new(&self.tasks, &self.filter, self.selected_index);
        frame.render_widget(widget, area);
    }

    /// Render log view widget
    fn render_log_view(&self, frame: &mut Frame, area: Rect) {
        if !self.tasks.is_empty() && self.selected_index < self.tasks.len() {
            let selected_task = &self.tasks[self.selected_index];
            let widget = LogViewerWidget::with_scroll_offset(selected_task, self.log_scroll_offset);
            frame.render_widget(widget, area);
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}
