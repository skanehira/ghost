use crate::app::{
    process,
    storage::{Task, TaskStatus},
};
use std::fs::OpenOptions;
use std::io::Write;

/// Check and update the status of a single task based on process existence
pub fn update_task_status_if_needed(task: &mut Task) -> bool {
    if task.status == TaskStatus::Running && !process::exists(task.pid) {
        task.status = TaskStatus::Exited;
        task.finished_at = Some(crate::app::helpers::now_timestamp());
        
        // Write execution time info to log file
        if let Err(e) = write_execution_summary_to_log(task) {
            eprintln!("Failed to write execution summary to log: {e}");
        }
        
        true // Status was updated
    } else {
        false // Status was not updated
    }
}

/// Write execution summary to the task's log file
pub fn write_execution_summary_to_log(task: &Task) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(&task.log_path)?;
    
    writeln!(file, "\n{}", "=".repeat(50))?;
    writeln!(file, "Task completed: {}", task.id)?;
    writeln!(file, "Exit status: {}", task.status)?;
    writeln!(file, "Duration: {}", task.duration_string())?;
    writeln!(file, "Started at: {}", task.started_at_datetime().format("%Y-%m-%d %H:%M:%S"))?;
    if let Some(finished_dt) = task.finished_at_datetime() {
        writeln!(file, "Finished at: {}", finished_dt.format("%Y-%m-%d %H:%M:%S"))?;
    }
    writeln!(file, "{}", "=".repeat(50))?;
    
    Ok(())
}

/// Determine task status based on process state
pub fn determine_task_status(pid: u32) -> TaskStatus {
    if process::exists(pid) {
        TaskStatus::Running
    } else {
        TaskStatus::Exited
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::storage::Task;

    #[test]
    fn test_update_task_status_if_needed_running_nonexistent() {
        let mut task = Task {
            id: "test".to_string(),
            pid: 99999, // Non-existent PID
            pgid: None,
            command: "[]".to_string(),
            env: None,
            cwd: None,
            status: TaskStatus::Running,
            exit_code: None,
            started_at: 0,
            finished_at: None,
            log_path: "/tmp/test.log".to_string(),
        };

        let updated = update_task_status_if_needed(&mut task);
        assert!(updated);
        assert_eq!(task.status, TaskStatus::Exited);
        assert!(task.finished_at.is_some());
    }

    #[test]
    fn test_update_task_status_if_needed_already_exited() {
        let mut task = Task {
            id: "test".to_string(),
            pid: 1, // Likely existing PID
            pgid: None,
            command: "[]".to_string(),
            env: None,
            cwd: None,
            status: TaskStatus::Exited,
            exit_code: None,
            started_at: 0,
            finished_at: None,
            log_path: "/tmp/test.log".to_string(),
        };

        let updated = update_task_status_if_needed(&mut task);
        assert!(!updated);
        assert_eq!(task.status, TaskStatus::Exited);
    }
}
