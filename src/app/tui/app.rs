use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{layout::Rect, Frame};
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs;
use std::process::Child;
use std::time::SystemTime;
use tui_scrollview::ScrollViewState;

use super::log_viewer_scrollview::LogViewerScrollWidget;
use super::table_state_scroll::TableScroll;
use super::{ConfirmationDialog, SearchType, TaskFilter, ViewMode};
use crate::app::config::Config;
use crate::app::error::Result;
use crate::app::storage;
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
    pub log_scroll_state: ScrollViewState,
    pub selected_task_id: Option<String>,
    pub env_scroll_state: ScrollViewState,
    pub last_render_area: Rect,
    conn: Connection,
    log_cache: HashMap<String, LogCache>,
    child_processes: HashMap<String, Child>,
    // Cache for web server port info keyed by PID to avoid expensive lookups in render
    pub port_cache: HashMap<u32, String>,
    pub search_query: String,
    pub previous_view_mode: ViewMode,    // 検索モードから戻るため
    pub filtered_tasks: Vec<Task>,       // フィルタリング済みタスク
    pub is_search_filtered: bool,        // 検索フィルタリング中かどうか
    pub current_log_task: Option<Task>,  // ログビュー中の選択されたタスク
    pub search_type: Option<SearchType>, // 検索のタイプ
    pub confirmation_dialog: Option<ConfirmationDialog>, // 確認ダイアログの状態
    pub log_auto_scroll: bool,           // ログの自動スクロール機能（tail -f モード）
    // 非Runningフィルタ用の表示期間（時間）。Noneの場合は既定(24h)を使う
    non_running_window_hours: u64,
}

impl TuiApp {
    pub fn new() -> Result<Self> {
        let conn = storage::init_database()?;

        Ok(Self {
            tasks: Vec::new(),
            table_scroll: TableScroll::new(),
            // Default to showing only running tasks
            filter: TaskFilter::Running,
            should_quit: false,
            view_mode: ViewMode::TaskList,
            log_scroll_offset: 0,
            log_lines_count: 0,
            log_scroll_state: ScrollViewState::default(),
            selected_task_id: None,
            env_scroll_state: ScrollViewState::default(),
            last_render_area: Rect::default(),
            conn,
            log_cache: HashMap::new(),
            child_processes: HashMap::new(),
            port_cache: HashMap::new(),
            search_query: String::new(),
            previous_view_mode: ViewMode::TaskList,
            filtered_tasks: Vec::new(),
            is_search_filtered: false,
            current_log_task: None,
            search_type: None,
            confirmation_dialog: None,
            log_auto_scroll: false,
            non_running_window_hours: 24,
        })
    }

    /// Create a new TuiApp with a specific config (for testing)
    pub fn new_with_config(config: Config) -> Result<Self> {
        let conn = storage::init_database_with_config(Some(config))?;

        Ok(Self {
            tasks: Vec::new(),
            table_scroll: TableScroll::new(),
            // Default to showing only running tasks
            filter: TaskFilter::Running,
            should_quit: false,
            view_mode: ViewMode::TaskList,
            log_scroll_offset: 0,
            log_lines_count: 0,
            log_scroll_state: ScrollViewState::default(),
            selected_task_id: None,
            env_scroll_state: ScrollViewState::default(),
            last_render_area: Rect::default(),
            conn,
            log_cache: HashMap::new(),
            child_processes: HashMap::new(),
            port_cache: HashMap::new(),
            search_query: String::new(),
            previous_view_mode: ViewMode::TaskList,
            filtered_tasks: Vec::new(),
            is_search_filtered: false,
            current_log_task: None,
            search_type: None,
            confirmation_dialog: None,
            log_auto_scroll: false,
            non_running_window_hours: 24,
        })
    }

    /// Create with custom day window (days * 25h) for non-running filters
    pub fn new_with_day_window(day_window: Option<u64>) -> Result<Self> {
        let mut app = Self::new()?;
        if let Some(d) = day_window {
            // 1日=25時間指定
            app.non_running_window_hours = d.saturating_mul(25);
        }
        Ok(app)
    }

    /// Load tasks from database
    pub fn refresh_tasks(&mut self) -> Result<()> {
        // Clean up finished child processes first
        self.cleanup_finished_processes();

        // Filter status for database query
        let status_filter = match self.filter {
            TaskFilter::All => None,
            TaskFilter::Running => Some("running"),
            TaskFilter::Exited => Some("exited"),
            TaskFilter::Killed => Some("killed"),
        };

        // Running は全期間。他は指定ウィンドウ(既定24h、または -d 指定の25h×日数)
        if matches!(self.filter, TaskFilter::Running) {
            self.tasks = task_repository::get_tasks_with_process_check(&self.conn, status_filter, true)?;
        } else {
            // 現在時刻からウィンドウ分を差し引いた閾値を計算
            let since = chrono::Utc::now()
                .checked_sub_signed(chrono::Duration::hours(self.non_running_window_hours as i64))
                .unwrap()
                .timestamp();
            self.tasks = task_repository::get_tasks_with_process_check_since(
                &self.conn,
                status_filter,
                since,
            )?;
        }

        // Update (throttled) port cache for running tasks only to avoid heavy work in render.
        // Strategy: populate cache entries for any running PID missing in cache; remove stale PIDs.
        // We intentionally avoid refreshing existing entries every tick to reduce lsof/ps calls.
        use std::collections::HashSet;
        let current_running_pids: HashSet<u32> = self
            .tasks
            .iter()
            .filter(|t| t.status == crate::app::storage::task_status::TaskStatus::Running)
            .map(|t| t.pid)
            .collect();

        // Remove cache entries for PIDs that are no longer in the list
        self.port_cache
            .retain(|pid, _| current_running_pids.contains(pid));

        // Fill cache for missing PIDs only (one-shot; avoids blocking keystrokes)
        for pid in current_running_pids {
            if !self.port_cache.contains_key(&pid) {
                let port = crate::app::helpers::extract_web_server_info(pid)
                    .unwrap_or_else(|| "-".to_string());
                self.port_cache.insert(pid, port);
            }
        }

        // Update search filter if active
        if self.is_search_filtered || !self.search_query.is_empty() || self.search_type.is_some() {
            self.update_search_filter();
        }

        // Update table scroll with new item count
        let display_tasks = self.get_display_tasks();
        self.table_scroll.set_total_items(display_tasks.len());

        Ok(())
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.view_mode {
            ViewMode::TaskList => self.handle_task_list_key(key),
            ViewMode::LogView => self.handle_log_view_key(key),
            ViewMode::ProcessDetails => self.handle_process_details_key(key),
            ViewMode::SearchProcessName | ViewMode::SearchLogContent | ViewMode::SearchInLog => {
                self.handle_search_key(key)
            }
            ViewMode::ConfirmationDialog => self.handle_confirmation_dialog_key(key),
        }
    }

    fn handle_task_list_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => {
                if self.is_search_filtered {
                    // Clear search filter
                    self.search_query.clear();
                    self.is_search_filtered = false;
                    self.filtered_tasks.clear();
                    self.search_type = None;
                    self.table_scroll = TableScroll::new();
                    self.table_scroll.set_total_items(self.tasks.len());
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Esc => {
                if self.is_search_filtered {
                    // Clear search filter with Esc key
                    self.search_query.clear();
                    self.is_search_filtered = false;
                    self.filtered_tasks.clear();
                    self.search_type = None;
                    self.table_scroll = TableScroll::new();
                    self.table_scroll.set_total_items(self.tasks.len());
                } else {
                    self.should_quit = true;
                }
            }
            KeyCode::Char('j') => {
                self.table_scroll.next();
            }
            KeyCode::Char('k') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.table_scroll.previous();
            }
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::NONE) => {
                self.table_scroll.first();
            }
            KeyCode::Char('G') => {
                self.table_scroll.last();
            }
            KeyCode::Enter => {
                let display_tasks = self.get_display_tasks();
                if !display_tasks.is_empty() {
                    self.view_mode = ViewMode::LogView;
                    self.initialize_log_view();
                }
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('s') => {
                if !self.tasks.is_empty() {
                    self.stop_task(false);
                }
            }
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if !self.tasks.is_empty() {
                    self.stop_task(true);
                }
            }
            KeyCode::Tab => {
                self.cycle_filter();
                self.refresh_tasks()?;
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let page_size = self.calculate_table_page_size();
                self.table_scroll.page_down(page_size);
            }
            KeyCode::Char('d') => {
                if !self.tasks.is_empty() {
                    let display_tasks = self.get_display_tasks();
                    if !display_tasks.is_empty() {
                        let selected_task = &display_tasks[self.selected_index()];
                        self.selected_task_id = Some(selected_task.id.clone());
                        self.view_mode = ViewMode::ProcessDetails;
                        self.env_scroll_state = ScrollViewState::default();
                    }
                }
            }
            KeyCode::Char('o') => {
                if !self.tasks.is_empty() {
                    let display_tasks = self.get_display_tasks();
                    if !display_tasks.is_empty() {
                        let selected_task = &display_tasks[self.selected_index()];
                        // Only open browser for running web servers
                        if selected_task.status
                            == crate::app::storage::task_status::TaskStatus::Running
                        {
                            self.open_browser_for_task(selected_task);
                        }
                    }
                }
            }
            KeyCode::Char('r') if key.modifiers.is_empty() => {
                if !self.tasks.is_empty() {
                    let display_tasks = self.get_display_tasks();
                    if !display_tasks.is_empty() {
                        let selected_task = &display_tasks[self.selected_index()];
                        self.show_restart_confirmation(selected_task);
                    }
                }
            }
            KeyCode::Char('R') => {
                self.rerun_selected_command()?;
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let page_size = self.calculate_table_page_size();
                self.table_scroll.page_up(page_size);
            }
            KeyCode::Char('/') => {
                self.search_query.clear();
                self.previous_view_mode = self.view_mode.clone();
                self.view_mode = ViewMode::SearchProcessName;
                self.search_type = Some(SearchType::ProcessName);
            }
            // Ctrl+G log content search temporarily disabled (not yet implemented)
            // KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) && !self.tasks.is_empty() => {
            //     self.search_query.clear();
            //     self.previous_view_mode = self.view_mode.clone();
            //     self.view_mode = ViewMode::SearchLogContent;
            //     self.search_type = Some(SearchType::LogContent);
            // }
            _ => {}
        }

        Ok(())
    }

    fn handle_log_view_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                // Return to task list and restore search filter state if needed
                self.view_mode = ViewMode::TaskList;
                // If we have a search query, ensure search filtered state is restored
                if !self.search_query.is_empty() {
                    self.is_search_filtered = true;
                }
                self.log_scroll_state.scroll_to_top();
                // Clear the current log task
                self.current_log_task = None;
                // Reset auto-scroll
                self.log_auto_scroll = false;
            }
            KeyCode::Char('j') => {
                self.log_scroll_state.scroll_down();
                // Manual scrolling disables auto-scroll
                self.log_auto_scroll = false;
            }
            KeyCode::Char('k') => {
                self.log_scroll_state.scroll_up();
                // Manual scrolling disables auto-scroll
                self.log_auto_scroll = false;
            }
            KeyCode::Char('h') => {
                self.log_scroll_state.scroll_left();
            }
            KeyCode::Char('l') => {
                self.log_scroll_state.scroll_right();
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Scroll down multiple lines (half page)
                for _ in 0..10 {
                    self.log_scroll_state.scroll_down();
                }
                // Manual scrolling disables auto-scroll
                self.log_auto_scroll = false;
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Scroll up multiple lines (half page)
                for _ in 0..10 {
                    self.log_scroll_state.scroll_up();
                }
                // Manual scrolling disables auto-scroll
                self.log_auto_scroll = false;
            }
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::NONE) => {
                self.log_scroll_state.scroll_to_top();
                // Manual scrolling disables auto-scroll
                self.log_auto_scroll = false;
            }
            KeyCode::Char('G') => {
                self.log_scroll_state.scroll_to_bottom();
                // When jumping to bottom, could enable auto-scroll
                // but let's keep it manual for now
            }
            KeyCode::Char('f') => {
                // Toggle auto-scroll mode (like tail -f)
                self.log_auto_scroll = !self.log_auto_scroll;
                if self.log_auto_scroll {
                    // When enabling auto-scroll, jump to bottom
                    self.log_scroll_state.scroll_to_bottom();
                }
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('d') => {
                // Switch to process details for the same task
                if let Some(ref current_task) = self.current_log_task {
                    self.selected_task_id = Some(current_task.id.clone());
                    self.view_mode = ViewMode::ProcessDetails;
                    self.env_scroll_state = ScrollViewState::default();
                }
            }
            KeyCode::Char('/') => {
                self.search_query.clear();
                self.previous_view_mode = self.view_mode.clone();
                self.view_mode = ViewMode::SearchInLog;
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_process_details_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                // Return to task list and restore search filter state if needed
                self.view_mode = ViewMode::TaskList;
                // If we have a search query, ensure search filtered state is restored
                if !self.search_query.is_empty() {
                    self.is_search_filtered = true;
                }
                self.env_scroll_state = ScrollViewState::default();
            }
            KeyCode::Char('j') => {
                self.env_scroll_state.scroll_down();
            }
            KeyCode::Char('k') => {
                self.env_scroll_state.scroll_up();
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('c') => {
                // Copy command to clipboard (macOS)
                if let Some(task_id) = &self.selected_task_id {
                    let display_tasks = self.get_display_tasks();
                    if let Some(task) = display_tasks.iter().find(|t| t.id == *task_id) {
                        let command = self.parse_command(&task.command);
                        let _ = std::process::Command::new("pbcopy")
                            .stdin(std::process::Stdio::piped())
                            .spawn()
                            .and_then(|mut child| {
                                use std::io::Write;
                                if let Some(stdin) = child.stdin.as_mut() {
                                    stdin.write_all(command.as_bytes())?;
                                }
                                child.wait()
                            });
                    }
                }
            }
            KeyCode::Char('l') => {
                // Switch to log view for the same task
                if let Some(task_id) = &self.selected_task_id {
                    let display_tasks = self.get_display_tasks();
                    if let Some(task) = display_tasks.iter().find(|t| t.id == *task_id) {
                        self.current_log_task = Some(task.clone());
                        self.view_mode = ViewMode::LogView;
                        self.initialize_log_view();
                    }
                }
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.env_scroll_state.scroll_page_down();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.env_scroll_state.scroll_page_up();
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                // Cancel search and return to previous view
                self.search_query.clear();
                self.is_search_filtered = false;
                self.filtered_tasks.clear();
                self.search_type = None;
                self.view_mode = self.previous_view_mode.clone();
                self.table_scroll = TableScroll::new();
                self.table_scroll.set_total_items(self.tasks.len());
            }
            KeyCode::Enter => {
                // Open log view for selected task
                let display_tasks = self.get_display_tasks();
                if !display_tasks.is_empty() {
                    self.view_mode = ViewMode::LogView;
                    self.initialize_log_view();
                }
            }
            KeyCode::Tab => {
                // Only confirm search if there's actually a search query
                if !self.search_query.is_empty() {
                    self.is_search_filtered = true;
                    self.view_mode = ViewMode::TaskList;
                    // Keep current selection position, just update total items
                    self.table_scroll
                        .set_total_items(self.get_display_tasks().len());
                }
                // If search query is empty, do nothing (don't change modes)
            }
            // Navigation in search mode with Ctrl-n/p and Ctrl-j/k
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.table_scroll.next();
            }
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.table_scroll.previous();
            }
            KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.table_scroll.next();
            }
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.table_scroll.previous();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                // Update filtering immediately
                self.update_search_filter();
                // Reset selection to first item when filter changes
                self.table_scroll.set_total_items(self.filtered_tasks.len());
                if !self.filtered_tasks.is_empty() {
                    self.table_scroll.select(Some(0));
                }
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.search_query.push(c);
                // Update filtering immediately
                self.update_search_filter();
                // Reset selection to first item when filter changes
                self.table_scroll.set_total_items(self.filtered_tasks.len());
                if !self.filtered_tasks.is_empty() {
                    self.table_scroll.select(Some(0));
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn initialize_log_view(&mut self) {
        if let Some(selected) = self.table_scroll.selected() {
            let display_tasks = self.get_display_tasks();
            if selected < display_tasks.len() {
                let selected_task = &display_tasks[selected];

                // Save the selected task for log view
                self.current_log_task = Some(selected_task.clone());
                let log_path = &selected_task.log_path;

                if let Some(cache) = self.log_cache.get(log_path) {
                    self.log_lines_count = cache.content.len();
                } else {
                    self.log_lines_count = 0;
                }

                // Reset scroll state to start from the top
                self.log_scroll_state.scroll_to_top();

                // Reset auto-scroll when opening log view
                self.log_auto_scroll = false;
            }
        }
    }


    /// Render the TUI
    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        self.last_render_area = area;
        match self.view_mode {
            ViewMode::TaskList => self.render_task_list(frame, area),
            ViewMode::LogView => self.render_log_view(frame, area),
            ViewMode::ProcessDetails => self.render_process_details(frame, area),
            ViewMode::SearchProcessName | ViewMode::SearchLogContent | ViewMode::SearchInLog => {
                self.render_search_mode(frame, area)
            }
            ViewMode::ConfirmationDialog => self.render_confirmation_dialog(frame, area),
        }
    }

    /// Render task list widget
    fn render_task_list(&mut self, frame: &mut Frame, area: Rect) {
        use super::task_list::TaskListWidget;

        let display_tasks = self.get_display_tasks();
        let widget = if self.is_search_filtered && !self.search_query.is_empty() {
            TaskListWidget::with_search(
                display_tasks,
                &self.filter,
                &mut self.table_scroll,
                &self.port_cache,
                self.search_query.clone(),
            )
        } else {
            TaskListWidget::new(display_tasks, &self.filter, &mut self.table_scroll, &self.port_cache)
        };
        frame.render_widget(widget, area);
    }

    /// Render search mode UI
    fn render_search_mode(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            style::{Color, Modifier, Style},
            widgets::{Block, Borders, Paragraph},
        };

        // Split area: search bar at bottom, content above
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(4)].as_ref())
            .split(area);

        // Render task list with search results
        use super::task_list::TaskListWidget;

        let display_tasks = self.get_display_tasks();
        let widget = TaskListWidget::with_search(
            display_tasks,
            &self.filter,
            &mut self.table_scroll,
            &self.port_cache,
            self.search_query.clone(),
        );
        frame.render_widget(widget, chunks[0]);

        // Render search input at bottom with help text
        let (search_title, help_text) = match self.view_mode {
            ViewMode::SearchProcessName => (
                "Search Process Name",
                " Enter:Log  Tab:Execute  C-n/p/j/k:Move  Esc:Cancel",
            ),
            ViewMode::SearchLogContent => (
                "Search in Logs (grep)",
                " Enter:Log  Tab:Execute  C-n/p/j/k:Move  Esc:Cancel",
            ),
            ViewMode::SearchInLog => (
                "Search in Current Log",
                " Enter:Log  Tab:Execute  C-n/p/j/k:Move  Esc:Cancel",
            ),
            _ => ("Search", " Enter:Log  Tab:Execute  Esc:Cancel"),
        };

        // Split search area: input box + help line
        let search_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(1)].as_ref())
            .split(chunks[1]);

        let search_block = Block::default()
            .borders(Borders::ALL)
            .title(search_title)
            .style(Style::default().fg(Color::Yellow));

        let search_text = Paragraph::new(self.search_query.as_str())
            .block(search_block)
            .style(Style::default().add_modifier(Modifier::BOLD));

        frame.render_widget(search_text, search_chunks[0]);

        // Render help text with current match count
        let match_count = if matches!(
            self.view_mode,
            ViewMode::SearchProcessName | ViewMode::SearchLogContent
        ) {
            format!(" {} matches  {}", self.filtered_tasks.len(), help_text)
        } else {
            help_text.to_string()
        };
        let help_paragraph =
            Paragraph::new(match_count).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(help_paragraph, search_chunks[1]);

        // Set cursor position (inside border, after text)
        frame.set_cursor_position((
            search_chunks[0].x + 1 + self.search_query.len() as u16,
            search_chunks[0].y + 1,
        ));
    }

    /// Render log view widget
    fn render_log_view(&mut self, frame: &mut Frame, area: Rect) {

        if let Some(ref selected_task) = self.current_log_task {
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

            // Use scrollview widget
            let scrollview_widget = match update_strategy {
                UpdateStrategy::FullReload => {
                    LogViewerScrollWidget::new(selected_task, self.log_auto_scroll)
                }
                UpdateStrategy::Incremental(previous_size) => {
                    let cache = self.log_cache.get(log_path).unwrap();
                    LogViewerScrollWidget::load_incremental_content(
                        selected_task,
                        cache.content.clone(),
                        previous_size,
                        self.log_auto_scroll,
                    )
                }
                UpdateStrategy::UseCache => {
                    let cache = self.log_cache.get(log_path).unwrap();
                    LogViewerScrollWidget::with_cached_content(
                        selected_task,
                        cache.content.clone(),
                        self.log_auto_scroll,
                    )
                }
            };

            // Update cache if needed
            if matches!(
                update_strategy,
                UpdateStrategy::FullReload | UpdateStrategy::Incremental(_)
            ) {
                if let Ok(metadata) = fs::metadata(log_path) {
                    let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    self.log_cache.insert(
                        log_path.clone(),
                        LogCache {
                            content: scrollview_widget.get_lines().to_vec(),
                            last_modified: modified,
                            file_size: metadata.len(),
                        },
                    );
                }
            }

            // Update line count
            let new_lines_count = scrollview_widget.get_lines_count();
            let lines_added = new_lines_count > self.log_lines_count;
            self.log_lines_count = new_lines_count;

            // Auto-scroll to bottom if enabled and new content was added
            if self.log_auto_scroll && lines_added {
                self.log_scroll_state.scroll_to_bottom();
            }

            // Render with scrollview state
            frame.render_stateful_widget(scrollview_widget, area, &mut self.log_scroll_state);

            // Show auto-scroll indicator
            if self.log_auto_scroll {
                use ratatui::{
                    style::{Color, Style},
                    text::{Line, Span},
                    widgets::Paragraph,
                };

                // Create a small indicator in the top-right corner
                let indicator_width = 14; // " [Auto-Scroll] "
                let indicator_height = 1;
                let x = area.right().saturating_sub(indicator_width + 1);
                let y = area.y + 1;

                if x > area.x && y < area.bottom() {
                    let indicator_area = Rect::new(x, y, indicator_width, indicator_height);

                    let indicator = Paragraph::new(Line::from(vec![
                        Span::styled(" ", Style::default()),
                        Span::styled("[Auto-Scroll]", Style::default().fg(Color::Yellow)),
                    ]));

                    frame.render_widget(indicator, indicator_area);

                }


            }

            // Update cache if needed
            if matches!(
                update_strategy,
                UpdateStrategy::FullReload | UpdateStrategy::Incremental(_)
            ) && let Ok(metadata) = fs::metadata(log_path)
            {
                let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                self.log_cache.insert(
                    log_path.clone(),
                    LogCache {
                        content: scrollview_widget.get_lines().to_vec(),
                        last_modified: modified,
                        file_size: metadata.len(),
                    },
                );
            }

            // Handle auto-scroll update
            let new_line_count = scrollview_widget.get_lines_count();
            self.handle_auto_scroll_update(&update_strategy, new_line_count);

            // Render with scrollview state
            frame.render_stateful_widget(scrollview_widget, area, &mut self.log_scroll_state);
        }
    }

    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Stop the selected task
    fn stop_task(&mut self, force: bool) {
        let display_tasks = self.get_display_tasks();
        if self.selected_index() < display_tasks.len() {
            let task = &display_tasks[self.selected_index()];
            let task_id = &task.id;

            // Send signal to stop the task (commands::stop handles process group killing)
            // Use show_output=false to suppress console output in TUI
            let _ = crate::app::commands::stop(&self.conn, task_id, force, false);

            // Refresh task list to update status
            let _ = self.refresh_tasks();
        }
    }

    /// Cycle through task filters
    fn cycle_filter(&mut self) {
        self.filter = match self.filter {
            TaskFilter::All => TaskFilter::Running,
            TaskFilter::Running => TaskFilter::Exited,
            TaskFilter::Exited => TaskFilter::Killed,
            TaskFilter::Killed => TaskFilter::All,
        };
        // Reset selection when changing filter
        self.table_scroll = TableScroll::new();
    }

    /// Render process details view
    fn render_process_details(&mut self, frame: &mut Frame, area: Rect) {
        use super::process_details::ProcessDetailsWidget;

        // Find the selected task
        if let Some(task_id) = &self.selected_task_id {
            // Search in display tasks (could be filtered)
            let display_tasks = self.get_display_tasks();
            if let Some(task) = display_tasks.iter().find(|t| t.id == *task_id) {
                let widget = ProcessDetailsWidget::new(task);
                widget.render(frame, area, &mut self.env_scroll_state);
            } else {
                // Task not found, go back to task list
                self.view_mode = ViewMode::TaskList;
                self.selected_task_id = None;
            }
        } else {
            // No task selected, go back to task list
            self.view_mode = ViewMode::TaskList;
        }
    }

    /// Update search filtering based on current query
    fn update_search_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_tasks.clear();
            return;
        }

        let query = self.search_query.to_lowercase();
        self.filtered_tasks = self
            .tasks
            .iter()
            .filter(|task| {
                match self.search_type {
                    Some(SearchType::ProcessName) => {
                        // Parse command JSON and search in the readable command
                        let command = self.parse_command(&task.command);
                        command.to_lowercase().contains(&query)
                    }
                    Some(SearchType::LogContent) => {
                        // Log content search not yet implemented
                        false
                    }
                    None => false,
                }
            })
            .cloned()
            .collect();
    }

    /// Get display tasks (filtered or original)
    fn get_display_tasks(&self) -> Vec<Task> {
        // If we have a search query (either in search mode or confirmed search), use filtered tasks
        if !self.search_query.is_empty() || self.is_search_filtered {
            self.filtered_tasks.clone()
        } else {
            self.tasks.clone()
        }
    }

    /// Parse command from JSON format to readable string
    fn parse_command(&self, command_json: &str) -> String {
        match serde_json::from_str::<Vec<String>>(command_json) {
            Ok(cmd_vec) => cmd_vec.join(" "),
            Err(_) => command_json.to_string(),
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

    fn calculate_table_page_size(&self) -> usize {
        // Calculate the visible height of the table based on last render area
        // Account for borders (2), header (1), footer separator (1), and footer (1)
        let overhead = 5;
        self.last_render_area.height.saturating_sub(overhead) as usize
    }
    /// Extract web server info and open browser for selected task
    fn open_browser_for_task(&self, task: &Task) {
        if let Some(port_info) = crate::app::helpers::extract_web_server_info(task.pid) {
            let url = if port_info.starts_with(':') {
                format!("http://localhost{port_info}")
            } else {
                port_info
            };

            // Open browser with the URL (macOS)
            let _ = std::process::Command::new("open").arg(&url).spawn();
        }
    }

    /// Show restart/rerun confirmation dialog
    fn show_restart_confirmation(&mut self, task: &Task) {
        use super::{ConfirmationAction, ConfirmationDialog};

        // Parse command for display
        let command: Vec<String> = serde_json::from_str(&task.command).unwrap_or_default();
        let command_str = command.join(" ");

        // Determine action based on task status
        let action = match task.status {
            crate::app::storage::task_status::TaskStatus::Running => ConfirmationAction::Restart,
            _ => ConfirmationAction::Rerun,
        };

        self.confirmation_dialog = Some(ConfirmationDialog {
            action,
            task_id: task.id.clone(),
            task_command: command_str,
            selected_choice: false, // Default to No
        });

        self.view_mode = ViewMode::ConfirmationDialog;
    }

    /// Handle confirmation dialog key input
    fn handle_confirmation_dialog_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('h')
            | KeyCode::Left
            | KeyCode::Char('l')
            | KeyCode::Right
            | KeyCode::Char('j')
            | KeyCode::Char('k')
            | KeyCode::Char(' ')
            | KeyCode::Tab => {
                if let Some(ref mut dialog) = self.confirmation_dialog {
                    dialog.selected_choice = !dialog.selected_choice; // Toggle
                }
            }
            KeyCode::Enter => {
                if let Some(dialog) = self.confirmation_dialog.take() {
                    if dialog.selected_choice {
                        // User chose Yes
                        self.execute_restart_or_rerun(dialog)?;
                    }
                }
                self.view_mode = ViewMode::TaskList;
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.confirmation_dialog = None;
                self.view_mode = ViewMode::TaskList;
            }
            _ => {}
        }
        Ok(())
    }

    /// Execute restart or rerun based on confirmation dialog
    fn execute_restart_or_rerun(&mut self, dialog: ConfirmationDialog) -> Result<()> {
        use crate::app::helpers;
        use std::time::Duration;

        // Parse command, cwd, and env from task
        let task = self.get_task_by_id(&dialog.task_id)?;
        let command: Vec<String> = serde_json::from_str(&task.command).unwrap_or_default();
        let cwd = task.cwd.clone();
        let env = task
            .env
            .as_ref()
            .and_then(|e| serde_json::from_str::<std::collections::HashMap<String, String>>(e).ok())
            .map(|map| {
                map.iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        match dialog.action {
            super::ConfirmationAction::Restart => {
                // Kill process and wait for termination (up to 5 seconds)
                let terminated = helpers::kill_and_wait(
                    task.pid,
                    task.pgid,
                    false, // Use SIGTERM first
                    Duration::from_secs(5),
                )?;

                if !terminated {
                    // If process didn't terminate, try SIGKILL
                    let _ = helpers::kill_and_wait(
                        task.pid,
                        task.pgid,
                        true, // Force kill
                        Duration::from_secs(2),
                    );
                }

                // Update task status in database
                let conn = crate::app::storage::init_database()?;
                crate::app::storage::update_task_status(
                    &conn,
                    &dialog.task_id,
                    crate::app::storage::TaskStatus::Killed,
                    None,
                )?;

                // Start the task again with original working directory and environment
                let cwd_path = cwd.map(std::path::PathBuf::from);
                match crate::app::commands::spawn(command, cwd_path, env) {
                    Ok(_) => {
                        // Success - the spawn function already verifies the process started
                    }
                    Err(e) => {
                        // Show error to user in a more visible way
                        eprintln!("❌ Failed to restart task: {e}");

                        // Optionally, you could store this error message to display in the UI
                        // For now, we'll just continue so the UI refreshes
                    }
                }
            }
            super::ConfirmationAction::Rerun => {
                // Run the command again with original working directory and environment
                let cwd_path = cwd.map(std::path::PathBuf::from);
                match crate::app::commands::spawn(command, cwd_path, env) {
                    Ok(_) => {
                        // Success - the spawn function already verifies the process started
                    }
                    Err(e) => {
                        // Show error to user in a more visible way
                        eprintln!("❌ Failed to rerun task: {e}");

                        // Optionally, you could store this error message to display in the UI
                        // For now, we'll just continue so the UI refreshes
                    }
                }
            }
        }

        // Refresh task list
        self.refresh_tasks()?;

        Ok(())
    }

    /// Rerun the selected task without confirmation (Shift+R shortcut)
    fn rerun_selected_command(&mut self) -> Result<()> {
        let display_tasks = self.get_display_tasks();
        if display_tasks.is_empty() {
            return Ok(());
        }

        let index = self.selected_index();
        if index >= display_tasks.len() {
            return Ok(());
        }

        let selected_task = &display_tasks[index];

        let command: Vec<String> = serde_json::from_str(&selected_task.command).map_err(|e| {
            crate::app::error::GhostError::InvalidArgument {
                message: format!("Failed to parse command JSON: {e}"),
            }
        })?;

        let env_pairs: Vec<(String, String)> = if let Some(env_json) = &selected_task.env {
            serde_json::from_str(env_json).map_err(|e| {
                crate::app::error::GhostError::InvalidArgument {
                    message: format!("Failed to parse environment JSON: {e}"),
                }
            })?
        } else {
            Vec::new()
        };

        let cwd = selected_task.cwd.as_ref().map(std::path::PathBuf::from);

        let (process_info, child) = crate::app::commands::spawn_with_shell_wrapper(
            command,
            cwd,
            env_pairs,
            &self.conn,
        )?;

        self.child_processes
            .insert(process_info.id.clone(), child);

        self.refresh_tasks()?;

        Ok(())
    }

    /// Clean up finished child processes to prevent zombies when rerunning tasks.
    fn cleanup_finished_processes(&mut self) {
        self.child_processes.retain(|_, child| match child.try_wait() {
            Ok(Some(_)) | Err(_) => false,
            Ok(None) => true,
        });
    }

    /// Get task by ID
    fn get_task_by_id(&self, task_id: &str) -> Result<&Task> {
        self.tasks.iter().find(|t| t.id == task_id).ok_or_else(|| {
            crate::app::error::GhostError::TaskNotFound {
                task_id: task_id.to_string(),
            }
        })
    }

    /// Render confirmation dialog
    fn render_confirmation_dialog(&mut self, frame: &mut Frame, area: Rect) {
        use ratatui::{
            layout::Alignment,
            style::{Color, Style},
            text::{Line, Span},
            widgets::{Block, Borders, Clear, Paragraph, Wrap},
        };

        // First render the task list as background
        self.render_task_list(frame, area);

        // Create a centered area for the dialog
        let dialog_area = popup_area(area, 70, 35);

        // Clear the dialog area
        frame.render_widget(Clear, dialog_area);

        if let Some(ref dialog) = self.confirmation_dialog {
            // Dialog title and action
            let action_text = match dialog.action {
                super::ConfirmationAction::Restart => "Restart",
                super::ConfirmationAction::Rerun => "Rerun",
            };

            let title = format!("{action_text} Task");

            // Create the dialog content
            let command_text = if dialog.task_command.len() > 50 {
                format!("{}...", &dialog.task_command[..47])
            } else {
                dialog.task_command.clone()
            };

            let content = vec![
                Line::from(""),
                Line::from(vec![
                    Span::raw("Are you sure you want to "),
                    Span::styled(
                        action_text.to_lowercase(),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(" this task?"),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Command: "),
                    Span::styled(command_text, Style::default().fg(Color::Cyan)),
                ]),
                Line::from(""),
                Line::from(""),
            ];

            // Create buttons
            let yes_style = if dialog.selected_choice {
                Style::default().fg(Color::Black).bg(Color::Green)
            } else {
                Style::default().fg(Color::Green)
            };

            let no_style = if !dialog.selected_choice {
                Style::default().fg(Color::Black).bg(Color::Red)
            } else {
                Style::default().fg(Color::Red)
            };

            let button_line = Line::from(vec![
                Span::raw("      "),
                Span::styled("[ Yes ]", yes_style),
                Span::raw("      "),
                Span::styled("[ No ]", no_style),
                Span::raw("      "),
            ]);

            let mut all_content = content;
            all_content.push(button_line);
            all_content.push(Line::from(""));
            all_content.push(Line::from(
                "h/j/k/l/Space: toggle | Enter: confirm | Esc: cancel",
            ));

            // Create the dialog widget
            let dialog_widget = Paragraph::new(all_content)
                .block(Block::default().borders(Borders::ALL).title(title))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true });

            frame.render_widget(dialog_widget, dialog_area);
        }
    }
}

/// Create a centered popup area
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    use ratatui::layout::{Constraint, Direction, Layout};

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
