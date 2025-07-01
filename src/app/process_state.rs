use crate::app::{process, storage::TaskStatus};

/// Check and update the status of a single task based on process existence
pub fn update_task_status_if_needed(task: &mut crate::app::storage::Task) -> bool {
    if task.status == TaskStatus::Running && !process::exists(task.pid) {
        task.status = TaskStatus::Exited;
        task.finished_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        );
        true // Status was updated
    } else {
        false // Status was not updated
    }
}

/// Check if a process is still alive and matches expected status
pub fn verify_process_status(pid: u32, expected_status: TaskStatus) -> bool {
    match expected_status {
        TaskStatus::Running => process::exists(pid),
        TaskStatus::Exited | TaskStatus::Killed | TaskStatus::Unknown => !process::exists(pid),
    }
}

/// Get the appropriate task status based on process existence and force flag
pub fn determine_status_after_kill(force: bool) -> TaskStatus {
    if force {
        TaskStatus::Killed
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

    #[test]
    fn test_determine_status_after_kill() {
        assert_eq!(determine_status_after_kill(true), TaskStatus::Killed);
        assert_eq!(determine_status_after_kill(false), TaskStatus::Exited);
    }
}
