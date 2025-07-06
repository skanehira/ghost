#[derive(Debug, thiserror::Error)]
pub enum GhostError {
    // Process-related errors
    #[error("Process spawn failed: {message}")]
    ProcessSpawn { message: String },

    #[error("Process operation failed: {message}")]
    ProcessOperation { message: String },

    #[error("Log file creation failed: {path} - {source}")]
    LogFileCreation {
        path: String,
        #[source]
        source: std::io::Error,
    },

    // Storage-related errors
    #[error("Database error: {source}")]
    Database {
        #[from]
        source: rusqlite::Error,
    },

    #[error("Data serialization error: {source}")]
    Serialization {
        #[from]
        source: serde_json::Error,
    },

    // File system errors
    #[error("File operation failed: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    // Task management errors
    #[error("Task not found: {task_id}")]
    TaskNotFound { task_id: String },

    #[error("Task operation failed: {task_id} - {message}")]
    TaskOperation { task_id: String, message: String },

    // Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    // Input validation errors
    #[error("Invalid argument: {message}")]
    InvalidArgument { message: String },

    // System-level errors
    #[error("Unix system error: {source}")]
    Unix {
        #[from]
        source: nix::Error,
    },

    // File watching errors (for log following)
    #[error("File watching error: {message}")]
    FileWatch { message: String },
}

pub type Result<T> = std::result::Result<T, GhostError>;
