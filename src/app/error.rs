#[derive(Debug, thiserror::Error)]
pub enum GhostError {
    #[error("Process error: {0}")]
    Process(#[from] crate::app::process::ProcessError),

    #[error("Storage error: {0}")]
    Storage(#[from] crate::app::storage::StorageError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

impl From<&str> for GhostError {
    fn from(s: &str) -> Self {
        GhostError::InvalidArgument(s.to_string())
    }
}

impl From<String> for GhostError {
    fn from(s: String) -> Self {
        GhostError::InvalidArgument(s)
    }
}

pub type Result<T> = std::result::Result<T, GhostError>;
