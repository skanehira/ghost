use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{Frame, layout::Rect};
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs;
use std::time::SystemTime;

use super::log_viewer::LogViewerWidget;
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
    pub selected_index: usize,
    pub filter: TaskFilter,
    pub should_quit: bool,
    pub view_mode: ViewMode,
    pub log_scroll_offset: usize,
    pub log_lines_count: usize,
    pub table_scroll_offset: usize,
    conn: Connection,
    log_cache: HashMap<String, LogCache>,
}

impl TuiApp {
    pub fn new() -> Result<Self> {
        let config = Config::default();
        let conn = rusqlite::Connection::open(&config.db_path)?;

        Ok(Self {
            tasks: Vec::new(),
            selected_index: 0,
            filter: TaskFilter::All,
            should_quit: false,
            view_mode: ViewMode::TaskList,
            log_scroll_offset: 0,
            log_lines_count: 0,
            table_scroll_offset: 0,
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
                self.table_scroll_offset = 0;
            }
            KeyCode::Char('G') => {
                if !self.tasks.is_empty() {
                    self.selected_index = self.tasks.len() - 1;
                    self.adjust_scroll_for_selection();
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
            self.adjust_scroll_for_selection();
        }
    }

    fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.adjust_scroll_for_selection();
        }
    }

    fn adjust_scroll_for_selection(&mut self) {
        // Calculate visible area height (total height - header - footer - borders)
        // For simplicity, assume we can display about 5 task rows in the visible area
        let visible_rows = 5;

        // If selected item is below visible area, scroll down
        if self.selected_index >= self.table_scroll_offset + visible_rows {
            self.table_scroll_offset = self.selected_index.saturating_sub(visible_rows - 1);
        }
        // If selected item is above visible area, scroll up
        else if self.selected_index < self.table_scroll_offset {
            self.table_scroll_offset = self.selected_index;
        }
    }

    fn initialize_log_view(&mut self) {
        if !self.tasks.is_empty() && self.selected_index < self.tasks.len() {
            let selected_task = &self.tasks[self.selected_index];
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
    fn render_task_list(&self, frame: &mut Frame, area: Rect) {
        use super::task_list::TaskListWidget;

        let widget = TaskListWidget::new(&self.tasks, &self.filter, self.selected_index)
            .with_scroll_offset(self.table_scroll_offset);
        frame.render_widget(widget, area);
    }

    /// Render log view widget
    fn render_log_view(&mut self, frame: &mut Frame, area: Rect) {
        if !self.tasks.is_empty() && self.selected_index < self.tasks.len() {
            let selected_task = &self.tasks[self.selected_index];
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
                    let widget =
                        LogViewerWidget::with_scroll_offset(selected_task, self.log_scroll_offset);

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

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
}
