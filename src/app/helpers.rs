use std::path::PathBuf;

use rusqlite::Connection;

use crate::app::{error::Result, storage};

/// Initialize database connection with error handling
pub fn init_db_connection() -> Result<Connection> {
    Ok(storage::init_database()?)
}

/// Get a task by ID with standardized error handling
pub fn get_task_by_id(conn: &Connection, task_id: &str) -> Result<storage::Task> {
    Ok(storage::get_task(conn, task_id)?)
}

/// Get a task by ID and update its status if needed (process check)
pub fn get_task_with_status_update(conn: &Connection, task_id: &str) -> Result<storage::Task> {
    Ok(storage::update_task_status_by_process_check(conn, task_id)?)
}

/// Read file content with standardized error handling
pub fn read_file_content(file_path: &PathBuf) -> Result<String> {
    if !file_path.exists() {
        return Err(format!("File not found: {}", file_path.display()).into());
    }

    let content = std::fs::read_to_string(file_path)?;
    Ok(content)
}

/// Validate that a task is in running state
pub fn validate_task_running(task: &storage::Task) -> Result<()> {
    if task.status != storage::TaskStatus::Running {
        return Err(format!("Task {} is not running (status: {})", task.id, task.status).into());
    }
    Ok(())
}

/// Print file content to stdout (for log display)
pub fn print_file_content(content: &str) {
    print!("{content}");
}
