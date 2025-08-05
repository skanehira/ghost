use super::task_status::TaskStatus;
use chrono::{DateTime, Duration, Local, TimeZone};

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub pid: u32,
    pub pgid: Option<i32>,
    pub command: String,     // JSON serialized Vec<String>
    pub env: Option<String>, // JSON serialized environment variables
    pub cwd: Option<String>,
    pub status: TaskStatus,
    pub exit_code: Option<i32>,
    pub started_at: i64, // Unix timestamp
    pub finished_at: Option<i64>,
    pub log_path: String,
}

impl Task {
    /// Calculate the duration of the task execution
    pub fn duration(&self) -> Option<Duration> {
        match self.finished_at {
            Some(finished) => Some(Duration::seconds(finished - self.started_at)),
            None => {
                // If task is still running, calculate duration from start to now
                if self.status == TaskStatus::Running {
                    let now = Local::now().timestamp();
                    Some(Duration::seconds(now - self.started_at))
                } else {
                    None
                }
            }
        }
    }

    /// Get a human-readable duration string
    pub fn duration_string(&self) -> String {
        match self.duration() {
            Some(duration) => {
                let total_seconds = duration.num_seconds();
                if total_seconds < 60 {
                    format!("{}s", total_seconds)
                } else if total_seconds < 3600 {
                    let minutes = total_seconds / 60;
                    let seconds = total_seconds % 60;
                    format!("{}m{}s", minutes, seconds)
                } else if total_seconds < 86400 {
                    let hours = total_seconds / 3600;
                    let minutes = (total_seconds % 3600) / 60;
                    format!("{}h{}m", hours, minutes)
                } else {
                    let days = total_seconds / 86400;
                    let hours = (total_seconds % 86400) / 3600;
                    format!("{}d{}h", days, hours)
                }
            }
            None => "-".to_string(),
        }
    }

    /// Get the start time as a DateTime
    pub fn started_at_datetime(&self) -> DateTime<Local> {
        Local.timestamp_opt(self.started_at, 0).unwrap()
    }

    /// Get the finish time as a DateTime if available
    pub fn finished_at_datetime(&self) -> Option<DateTime<Local>> {
        self.finished_at
            .map(|ts| Local.timestamp_opt(ts, 0).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_for_completed_task() {
        let task = Task {
            id: "test-123".to_string(),
            pid: 1234,
            pgid: Some(1234),
            command: r#"["echo", "hello"]"#.to_string(),
            env: None,
            cwd: None,
            status: TaskStatus::Exited,
            exit_code: Some(0),
            started_at: 1700000000,
            finished_at: Some(1700003661), // 1 hour, 1 minute, 1 second later
            log_path: "/tmp/test.log".to_string(),
        };

        let duration = task.duration().unwrap();
        assert_eq!(duration.num_seconds(), 3661);
        assert_eq!(task.duration_string(), "1h1m");
    }

    #[test]
    fn test_duration_for_running_task() {
        let now = Local::now().timestamp();
        let task = Task {
            id: "test-456".to_string(),
            pid: 5678,
            pgid: Some(5678),
            command: r#"["sleep", "100"]"#.to_string(),
            env: None,
            cwd: None,
            status: TaskStatus::Running,
            exit_code: None,
            started_at: now - 150, // Started 150 seconds ago
            finished_at: None,
            log_path: "/tmp/test2.log".to_string(),
        };

        let duration = task.duration().unwrap();
        // Duration should be approximately 150 seconds
        assert!(duration.num_seconds() >= 149 && duration.num_seconds() <= 151);
    }

    #[test]
    fn test_duration_string_formats() {
        // Test seconds format
        let mut task = create_test_task();
        task.started_at = 1700000000;
        task.finished_at = Some(1700000045);
        assert_eq!(task.duration_string(), "45s");

        // Test minutes and seconds format
        task.finished_at = Some(1700000125);
        assert_eq!(task.duration_string(), "2m5s");

        // Test hours and minutes format
        task.finished_at = Some(1700007320);
        assert_eq!(task.duration_string(), "2h2m");

        // Test days and hours format
        task.finished_at = Some(1700093600);
        assert_eq!(task.duration_string(), "1d2h");
    }

    #[test]
    fn test_duration_for_non_running_unfinished_task() {
        let task = Task {
            id: "test-789".to_string(),
            pid: 9999,
            pgid: Some(9999),
            command: r#"["ls"]"#.to_string(),
            env: None,
            cwd: None,
            status: TaskStatus::Killed,
            exit_code: Some(-9),
            started_at: 1700000000,
            finished_at: None,
            log_path: "/tmp/test3.log".to_string(),
        };

        assert!(task.duration().is_none());
        assert_eq!(task.duration_string(), "-");
    }

    fn create_test_task() -> Task {
        Task {
            id: "test".to_string(),
            pid: 1000,
            pgid: Some(1000),
            command: r#"["test"]"#.to_string(),
            env: None,
            cwd: None,
            status: TaskStatus::Exited,
            exit_code: Some(0),
            started_at: 1700000000,
            finished_at: Some(1700001000),
            log_path: "/tmp/test.log".to_string(),
        }
    }
}
