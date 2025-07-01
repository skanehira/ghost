use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::Duration;

use crate::app::{error::Result, storage};

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

/// Follow a log file and print new lines as they appear (tail -f behavior)
pub fn follow_log_file(file_path: &PathBuf) -> Result<()> {
    if !file_path.exists() {
        return Err(format!("File not found: {}", file_path.display()).into());
    }

    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    // First, read and print existing content
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF reached
            Ok(_) => {
                print!("{line}");
                use std::io::Write;
                std::io::stdout().flush().unwrap_or(());
            }
            Err(e) => return Err(e.into()),
        }
    }

    // Now follow the file for new content
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // EOF reached, wait and try again
                std::thread::sleep(Duration::from_millis(100));

                // Re-seek to current position to check for new data
                let current_pos = reader.stream_position()?;
                let file = File::open(file_path)?;
                reader = BufReader::new(file);
                reader.seek(SeekFrom::Start(current_pos))?;
            }
            Ok(_) => {
                print!("{line}");
                // Flush stdout to ensure immediate output
                use std::io::Write;
                std::io::stdout().flush().unwrap_or(());
            }
            Err(e) => return Err(e.into()),
        }

        // Check for Ctrl+C or process termination
        // Note: In a real implementation, we'd want proper signal handling
        // For now, we'll rely on the parent process to kill us
    }
}
