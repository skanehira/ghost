use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Running,
    Exited,
    Killed,
    Unknown,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TaskStatus {
    /// Convert TaskStatus to string for database storage
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Running => "running",
            TaskStatus::Exited => "exited",
            TaskStatus::Killed => "killed",
            TaskStatus::Unknown => "unknown",
        }
    }

    /// Parse TaskStatus from string (for database retrieval)
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> TaskStatus {
        s.parse().unwrap_or(TaskStatus::Unknown)
    }
}

impl std::str::FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "running" => Ok(TaskStatus::Running),
            "exited" => Ok(TaskStatus::Exited),
            "killed" => Ok(TaskStatus::Killed),
            "unknown" => Ok(TaskStatus::Unknown),
            _ => Err(format!("Unknown task status: {s}")),
        }
    }
}
