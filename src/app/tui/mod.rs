pub mod app;
pub mod task_list;

use crate::app::storage::task::Task;

pub struct App {
    pub tasks: Vec<Task>,
    pub selected_index: usize,
    pub filter: TaskFilter,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskFilter {
    All,
    Running,
    Exited,
    Killed,
}

impl App {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            selected_index: 0,
            filter: TaskFilter::All,
        }
    }

    pub fn with_tasks(tasks: Vec<Task>) -> Self {
        Self {
            tasks,
            selected_index: 0,
            filter: TaskFilter::All,
        }
    }
}
