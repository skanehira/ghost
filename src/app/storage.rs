pub mod cleanup;
pub mod database;
pub mod error;
pub mod task;
pub mod task_repository;
pub mod task_status;

// Re-export for backward compatibility
pub use cleanup::{cleanup_old_tasks, cleanup_tasks_by_criteria, get_cleanup_candidates};
pub use database::init_database;
pub use error::{Result, StorageError};
pub use task::Task;
pub use task_repository::{
    delete_task, get_task, get_tasks, get_tasks_with_process_check, insert_task, row_to_task,
    update_task_status, update_task_status_by_process_check,
};
pub use task_status::TaskStatus;
