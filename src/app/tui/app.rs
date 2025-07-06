use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect};
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs;
use std::time::SystemTime;

use super::log_viewer::LogViewerWidget;
use super::table_state_scroll::TableScroll;
use super::{TaskFilter, ViewMode};
use crate::app::config::Config;
use crate::app::error::Result;
use crate::app::storage::task::Task;
use crate::app::storage::task_repository;

/// Cache for log file content
struct LogCache {
    content: Vec<String>,
    last_modified: SystemTime,
    file_size: u64,
}

enum UpdateStrategy {
    FullReload,
    Incremental(u64), // previous file size
    UseCache,
}

pub struct TuiApp {
    pub tasks: Vec<Task>,
    pub table_scroll: TableScroll,
    pub filter: TaskFilter,
    pub should_quit: bool,
    pub view_mode: ViewMode,
    pub log_scroll_offset: usize,
    pub log_lines_count: usize,
    conn: Connection,
    log_cache: HashMap<String, LogCache>,
}

impl TuiApp {
    pub fn new() -> Result<Self> {
        let config = Config::default();
        let conn = rusqlite::Connection::open(&config.db_path)?;

        Ok(Self {
            tasks: Vec::new(),
            table_scroll: TableScroll::new(),
            filter: TaskFilter::All,
            should_quit: false,
            view_mode: ViewMode::TaskList,
            log_scroll_offset: 0,
            log_lines_count: 0,
            conn,
            log_cache: HashMap::new(),
        })
    }

    /// Load tasks from database
    pub fn refresh_tasks(&mut self) -> Result<()> {
        // Filter status for database query
        let status_filter = match self.filter {
            TaskFilter::All => None,
            TaskFilter::Running => Some("running"),
            TaskFilter::Exited => Some("exited"),
            TaskFilter::Killed => Some("killed"),
        };

        self.tasks = task_repository::get_tasks_with_process_check(&self.conn, status_filter)?;

        // Update table scroll with new item count
        self.table_scroll.set_total_items(self.tasks.len());

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
            KeyCode::Char('k') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection_up();
            }
            KeyCode::Up => {
                self.move_selection_up();
            }

            // Go to top/bottom
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::NONE) => {
                // Handle 'gg' for top - this is simplified, real vim would need state tracking
                self.table_scroll.first();
            }
            KeyCode::Char('G') => {
                self.table_scroll.last();
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

            // Stop task with SIGTERM
            KeyCode::Char('s') => {
                if !self.tasks.is_empty() {
                    self.stop_task(false);
                }
            }

            // Kill task with SIGKILL
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if !self.tasks.is_empty() {
                    self.stop_task(true);
                }
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
            KeyCode::Char('k') => {
                self.scroll_log_up();
            }
            KeyCode::Up => {
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
        self.table_scroll.next();
    }

    fn move_selection_up(&mut self) {
        self.table_scroll.previous();
    }

    fn initialize_log_view(&mut self) {
        if let Some(selected) = self.table_scroll.selected() {
            if selected < self.tasks.len() {
                let selected_task = &self.tasks[selected];
                let log_path = &selected_task.log_path;

                // Check cache first
                if let Some(cache) = self.log_cache.get(log_path) {
                    self.log_lines_count = cache.content.len();
                } else {
                    // If not in cache, we'll load it on first render
                    self.log_lines_count = 0;
                }

                // Start at the beginning of the log
                self.log_scroll_offset = 0;
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
    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        match self.view_mode {
            ViewMode::TaskList => self.render_task_list(frame, area),
            ViewMode::LogView => self.render_log_view(frame, area),
        }
    }

    /// Render task list widget
    fn render_task_list(&mut self, frame: &mut Frame, area: Rect) {
        use super::task_list::TaskListWidget;

        let widget = TaskListWidget::new(&self.tasks, &self.filter, &mut self.table_scroll);
        frame.render_widget(widget, area);
    }

    /// Render log view widget
    fn render_log_view(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(selected) = self.table_scroll.selected() {
            if selected < self.tasks.len() {
                let selected_task = &self.tasks[selected];
                let log_path = &selected_task.log_path;

                // Check if we need to reload or incrementally update the file
                let update_strategy = if let Ok(metadata) = fs::metadata(log_path) {
                    let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    let file_size = metadata.len();

                    if let Some(cache) = self.log_cache.get(log_path) {
                        if modified > cache.last_modified {
                            if file_size > cache.file_size {
                                // File grew, use incremental update
                                UpdateStrategy::Incremental(cache.file_size)
                            } else {
                                // File changed in other ways, full reload
                                UpdateStrategy::FullReload
                            }
                        } else {
                            // No changes
                            UpdateStrategy::UseCache
                        }
                    } else {
                        // No cache exists, need to load
                        UpdateStrategy::FullReload
                    }
                } else {
                    // File doesn't exist or can't read metadata
                    UpdateStrategy::UseCache
                };

                let widget = match update_strategy {
                    UpdateStrategy::FullReload => {
                        // Load the file and update cache
                        let widget = LogViewerWidget::with_scroll_offset(
                            selected_task,
                            self.log_scroll_offset,
                        );

                        // Store in cache
                        if let Ok(metadata) = fs::metadata(log_path) {
                            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                            self.log_cache.insert(
                                log_path.clone(),
                                LogCache {
                                    content: widget.get_lines().to_vec(),
                                    last_modified: modified,
                                    file_size: metadata.len(),
                                },
                            );
                        }

                        widget
                    }
                    UpdateStrategy::Incremental(previous_size) => {
                        // Use incremental update
                        let cache = self.log_cache.get(log_path).unwrap();
                        let widget = LogViewerWidget::load_incremental_content(
                            selected_task,
                            self.log_scroll_offset,
                            cache.content.clone(),
                            previous_size,
                        );

                        // Update cache
                        if let Ok(metadata) = fs::metadata(log_path) {
                            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                            self.log_cache.insert(
                                log_path.clone(),
                                LogCache {
                                    content: widget.get_lines().to_vec(),
                                    last_modified: modified,
                                    file_size: metadata.len(),
                                },
                            );
                        }

                        widget
                    }
                    UpdateStrategy::UseCache => {
                        // Use cached content
                        let cache = self.log_cache.get(log_path).unwrap();
                        LogViewerWidget::with_cached_content(
                            selected_task,
                            self.log_scroll_offset,
                            cache.content.clone(),
                        )
                    }
                };

                // Update the internal line count for proper scrolling
                let new_lines_count = widget.get_lines_count();
                if new_lines_count != self.log_lines_count {
                    self.log_lines_count = new_lines_count;
                    // If we were at the bottom, stay at the bottom
                    if self.log_scroll_offset >= self.log_lines_count.saturating_sub(2) {
                        self.log_scroll_offset = self.log_lines_count.saturating_sub(1);
                    }
                }

                frame.render_widget(widget, area);
            }
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }


    /// Stop the selected task
    fn stop_task(&mut self, force: bool) {
        if self.selected_index() < self.tasks.len() {
            let task = &self.tasks[self.selected_index()];
            let task_id = &task.id;

            // Send signal to stop the task (commands::stop handles process group killing)
            let _ = crate::app::commands::stop(task_id, force);

            // Refresh task list to update status
            let _ = self.refresh_tasks();
        }
    }

    // Accessor methods for tests compatibility
    pub fn selected_index(&self) -> usize {
        self.table_scroll.selected().unwrap_or(0)
    }

    pub fn set_selected_index(&mut self, index: usize) {
        if index < self.tasks.len() {
            self.table_scroll.select(Some(index));
        }
    }

    pub fn table_scroll_offset(&self) -> usize {
        // Calculate visible offset based on selection
        let selected = self.table_scroll.selected().unwrap_or(0);
        selected.saturating_sub(2) // Keep some context above
    }

}
