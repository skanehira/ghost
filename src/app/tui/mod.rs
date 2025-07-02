pub mod app;
pub mod log_viewer;
pub mod task_list;

use crate::app::storage::task::Task;

pub struct App {
    pub tasks: Vec<Task>,
    pub selected_index: usize,
    pub filter: TaskFilter,
    pub table_scroll_offset: usize,
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
            table_scroll_offset: 0,
        }
    }

    pub fn with_tasks(tasks: Vec<Task>) -> Self {
        Self {
            tasks,
            selected_index: 0,
            filter: TaskFilter::All,
            table_scroll_offset: 0,
        }
    }

    pub fn with_tasks_and_scroll(tasks: Vec<Task>, scroll_offset: usize) -> Self {
        Self {
            tasks,
            selected_index: 0,
            filter: TaskFilter::All,
            table_scroll_offset: scroll_offset,
        }
    }
}
