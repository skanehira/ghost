use crate::app::error::Result;
use crate::app::process;
use std::time::{Duration, Instant};

/// Wait for a process to terminate with timeout
pub fn wait_for_process_termination(pid: u32, timeout: Duration) -> Result<bool> {
    let start = Instant::now();

    while start.elapsed() < timeout {
        if !process::exists(pid) {
            return Ok(true); // Process terminated successfully
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    Ok(false) // Timeout reached, process still running
}

/// Kill process and wait for termination
pub fn kill_and_wait(pid: u32, pgid: Option<i32>, force: bool, timeout: Duration) -> Result<bool> {
    // Kill process group if available, otherwise individual process
    if let Some(pgid) = pgid {
        process::kill_group(pgid, force)?;
    } else {
        process::kill(pid, force)?;
    }

    // Wait for process to terminate
    wait_for_process_termination(pid, timeout)
}

/// Wait for a process to start and verify it's running
pub fn wait_for_process_start(pid: u32, timeout: Duration) -> Result<bool> {
    let start = Instant::now();

    // First, give the process a moment to start
    std::thread::sleep(Duration::from_millis(100));

    while start.elapsed() < timeout {
        if process::exists(pid) {
            // Double-check after a short delay to ensure it's not immediately exiting
            std::thread::sleep(Duration::from_millis(100));
            if process::exists(pid) {
                return Ok(true); // Process is running
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    Ok(false) // Timeout reached or process exited
}

/// Verify log file exists and has content
pub fn verify_log_file(log_path: &str, timeout: Duration) -> Result<bool> {
    let start = Instant::now();

    while start.elapsed() < timeout {
        if let Ok(metadata) = std::fs::metadata(log_path) {
            if metadata.len() > 0 {
                return Ok(true);
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_wait_for_process_termination() {
        // Spawn a short-lived process
        let mut child = Command::new("sleep")
            .arg("0.1")
            .spawn()
            .expect("Failed to spawn process");

        let pid = child.id();

        // Process should terminate within 500ms
        let terminated = wait_for_process_termination(pid, Duration::from_millis(500)).unwrap();
        assert!(terminated);

        // Clean up zombie
        let _ = child.wait();
    }

    #[test]
    fn test_wait_for_process_termination_timeout() {
        // Spawn a long-lived process
        let mut child = Command::new("sleep")
            .arg("5")
            .spawn()
            .expect("Failed to spawn process");

        let pid = child.id();

        // Should timeout waiting for process
        let terminated = wait_for_process_termination(pid, Duration::from_millis(100)).unwrap();
        assert!(!terminated);

        // Clean up
        let _ = child.kill();
        let _ = child.wait();
    }
}
