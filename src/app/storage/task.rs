use super::task_status::TaskStatus;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
