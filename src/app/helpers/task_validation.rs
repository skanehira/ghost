use crate::app::{error, error::Result, storage};

/// Validate that a task is in running state
pub fn validate_task_running(task: &storage::Task) -> Result<()> {
    if task.status != storage::TaskStatus::Running {
        return Err(error::GhostError::TaskOperation {
            task_id: task.id.clone(),
            message: format!("Task is not running (status: {})", task.status),
        });
    }
    Ok(())
}
