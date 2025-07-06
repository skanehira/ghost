pub mod app;
pub mod log_viewer;
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
