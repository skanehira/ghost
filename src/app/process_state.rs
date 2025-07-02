use crate::app::{
    process,
    storage::{Task, TaskStatus},
};

/// Check and update the status of a single task based on process existence
pub fn update_task_status_if_needed(task: &mut Task) -> bool {
    if task.status == TaskStatus::Running && !process::exists(task.pid) {
        task.status = TaskStatus::Exited;
        task.finished_at = Some(crate::app::helpers::now_timestamp());
        true // Status was updated
    } else {
        false // Status was not updated
    }
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
