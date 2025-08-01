pub mod app;
pub mod log_viewer_scrollview;
pub mod process_details;
pub mod table_state_scroll;
pub mod task_list;

use self::table_state_scroll::TableScroll;
use crate::app::storage::task::Task;

pub struct App {
    pub tasks: Vec<Task>,
    pub selected_index: usize,
    pub filter: TaskFilter,
    pub table_scroll: TableScroll,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskFilter {
    All,
    Running,
    Exited,
    Killed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    TaskList,
    LogView,
    ProcessDetails,     // dキーでプロセス詳細表示
    SearchProcessName,  // /キーでプロセス名検索
    SearchLogContent,   // gキーでログ内容検索
    SearchInLog,        // ログビューで/キーでログ内検索
    ConfirmationDialog, // rキーで再起動/再実行確認ダイアログ
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchType {
    ProcessName, // プロセス名での検索
    LogContent,  // ログ内容での検索
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmationAction {
    Restart, // 再起動
    Rerun,   // 再実行
}

#[derive(Debug, Clone)]
pub struct ConfirmationDialog {
    pub action: ConfirmationAction,
    pub task_id: String,
    pub task_command: String,
    pub selected_choice: bool, // true: Yes, false: No (default)
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            selected_index: 0,
            filter: TaskFilter::All,
            table_scroll: TableScroll::new(),
        }
    }

    pub fn with_tasks(tasks: Vec<Task>) -> Self {
        let table_scroll = TableScroll::with_items(tasks.len());
        Self {
            tasks,
            selected_index: 0,
            filter: TaskFilter::All,
            table_scroll,
        }
    }

    pub fn with_tasks_and_scroll(tasks: Vec<Task>, scroll_offset: usize) -> Self {
        let mut table_scroll = TableScroll::with_items(tasks.len());
        if !tasks.is_empty() && scroll_offset < tasks.len() {
            table_scroll.select(Some(scroll_offset));
        }
        Self {
            tasks,
            selected_index: 0,
            filter: TaskFilter::All,
            table_scroll,
        }
    }
}
