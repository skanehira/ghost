use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_rolling_file::RollingFileAppenderBase;
use tracing_subscriber::fmt::format::FmtSpan;

/// Initialize file logger with rolling file appender
///
/// Creates a rolling file appender that:
/// - Writes to `ghost.log` in the specified directory
/// - Rotates when file size exceeds 5MB
/// - Keeps up to 5 historical log files
///
/// # Arguments
/// * `log_dir` - Directory where log files will be stored
///
/// # Returns
/// * `Some(WorkerGuard)` - Guard that must be held to ensure logs are flushed
/// * `None` - If logger initialization failed
pub fn init_file_logger(log_dir: &Path) -> Option<WorkerGuard> {
    // Create parent directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(log_dir) {
        eprintln!("Warning: Failed to create log directory: {e}");
        return None;
    }

    let log_file_path = log_dir.join("ghost.log");

    // Create rolling file appender with 5MB size limit
    let file_appender = RollingFileAppenderBase::builder()
        .filename(log_file_path.to_string_lossy().to_string())
        .max_filecount(5)
        .condition_max_file_size(5 * 1024 * 1024) // 5MB
        .build()
        .ok()?;

    // Get non-blocking writer
    let (non_blocking, guard) = file_appender.get_non_blocking_appender();

    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_span_events(FmtSpan::NONE)
        .with_target(false)
        .init();

    Some(guard)
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use tracing_rolling_file::RollingFileAppenderBase;

    #[test]
    fn test_rolling_file_appender_creates_log_file() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path();
        let log_file_path = log_dir.join("test.log");

        // Create rolling file appender
        let file_appender = RollingFileAppenderBase::builder()
            .filename(log_file_path.to_string_lossy().to_string())
            .max_filecount(5)
            .condition_max_file_size(5 * 1024 * 1024)
            .build();

        assert!(file_appender.is_ok(), "File appender should be created");

        // Write something to trigger file creation
        use std::io::Write;
        let mut appender = file_appender.unwrap();
        writeln!(appender, "test log message").unwrap();

        // Log file should be created
        assert!(log_file_path.exists(), "test.log should be created");
    }

    #[test]
    fn test_create_log_directory() {
        let temp_dir = tempdir().unwrap();
        let log_dir = temp_dir.path().join("nested").join("logs");

        // Directory should not exist yet
        assert!(!log_dir.exists());

        // Create directory
        std::fs::create_dir_all(&log_dir).unwrap();

        // Directory should be created
        assert!(log_dir.exists(), "Log directory should be created");
    }
}
