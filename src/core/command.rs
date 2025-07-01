use nix::unistd::setsid;
use std::fs::File;
use std::os::unix::process::CommandExt as _;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use uuid::Uuid;

#[cfg(unix)]
use nix::sys::signal::{self, Signal};
#[cfg(unix)]
use nix::unistd::Pid;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub id: String,
    pub pid: u32,
    #[cfg(unix)]
    pub pgid: i32,
    pub command: Vec<String>,
    pub log_path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Process spawn failed: {0}")]
    SpawnFailed(String),

    #[error("Failed to create log file: {0}")]
    LogFileCreation(String),

    #[cfg(unix)]
    #[error("Unix system error: {0}")]
    Unix(#[from] nix::Error),
}

pub type Result<T> = std::result::Result<T, CommandError>;

/// Get the default log directory path
fn get_log_dir() -> PathBuf {
    let base_dir = if cfg!(windows) {
        dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."))
    } else {
        dirs::data_dir().unwrap_or_else(|| PathBuf::from("."))
    };

    base_dir.join("ghost").join("logs")
}

/// Spawn a background process with logging
/// Returns both ProcessInfo and Child handle to allow proper cleanup
pub fn spawn_background_process(
    command: Vec<String>,
    log_dir: Option<PathBuf>,
) -> Result<(ProcessInfo, Child)> {
    if command.is_empty() {
        return Err(CommandError::SpawnFailed("Empty command".to_string()));
    }

    // Generate task ID and prepare paths
    let task_id = Uuid::new_v4().to_string();
    let log_dir = log_dir.unwrap_or_else(get_log_dir);

    // Create log directory if it doesn't exist
    std::fs::create_dir_all(&log_dir)?;

    let log_path = log_dir.join(format!("{task_id}.log"));

    // Create log file
    let log_file =
        File::create(&log_path).map_err(|e| CommandError::LogFileCreation(e.to_string()))?;

    // Setup command
    let mut cmd = Command::new(&command[0]);
    cmd.args(&command[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_file.try_clone()?))
        .stderr(Stdio::from(log_file));

    #[cfg(unix)]
    unsafe {
        cmd.pre_exec(|| setsid().map(|_| ()).map_err(std::io::Error::other));
    }

    // Spawn the process
    let child = cmd
        .spawn()
        .map_err(|e| CommandError::SpawnFailed(format!("Failed to spawn process: {e}")))?;

    let pid = child.id();

    // Get process group ID
    #[cfg(unix)]
    let pgid = {
        // The process group ID should be the same as PID after setsid()
        pid as i32
    };

    let info = ProcessInfo {
        id: task_id,
        pid,
        #[cfg(unix)]
        pgid,
        command,
        log_path,
    };

    Ok((info, child))
}

/// Check if a process is still running
pub fn process_exists(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // Send signal 0 to check if process exists
        // We need to check errno to distinguish between "no permission" and "no process"
        match signal::kill(Pid::from_raw(pid as i32), None) {
            Ok(_) => true,
            Err(nix::errno::Errno::ESRCH) => false, // No such process
            Err(_) => true, // Other errors (e.g., EPERM) mean process exists
        }
    }
}

/// Kill a process
pub fn kill_process(pid: u32, force: bool) -> Result<()> {
    #[cfg(unix)]
    {
        let signal = if force {
            Signal::SIGKILL
        } else {
            Signal::SIGTERM
        };
        signal::kill(Pid::from_raw(pid as i32), signal)?;
        Ok(())
    }
}

/// Kill a process group
#[cfg(unix)]
pub fn kill_process_group(pgid: i32, force: bool) -> Result<()> {
    let signal = if force {
        Signal::SIGKILL
    } else {
        Signal::SIGTERM
    };
    // Negative PID means process group
    signal::kill(Pid::from_raw(-pgid), signal)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_spawn_background_process() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_dir = temp_dir.path().to_path_buf();

        // Spawn a simple sleep command
        let command = vec!["sleep".to_string(), "2".to_string()];
        let result = spawn_background_process(command.clone(), Some(log_dir.clone()));

        assert!(result.is_ok());
        let (process_info, mut child) = result.unwrap();

        // Check process info
        assert!(!process_info.id.is_empty());
        assert!(process_info.pid > 0);
        assert_eq!(process_info.command, command);

        // Check log file exists
        assert!(process_info.log_path.exists());

        // Check process is running
        assert!(process_exists(process_info.pid));

        // Wait a bit and check again
        thread::sleep(Duration::from_millis(100));
        assert!(process_exists(process_info.pid));

        // Kill the process
        let _ = kill_process(process_info.pid, true);

        // Clean up zombie by waiting
        let _ = child.wait();
    }

    #[test]
    fn test_spawn_with_output() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_dir = temp_dir.path().to_path_buf();

        // Spawn echo command
        let command = vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo 'Hello, Ghost!' && echo 'Error message' >&2".to_string(),
        ];

        let result = spawn_background_process(command, Some(log_dir));
        assert!(result.is_ok());

        let (process_info, mut child) = result.unwrap();

        // Wait for process to complete
        let _ = child.wait();

        // Check log content
        let log_content = std::fs::read_to_string(&process_info.log_path).unwrap();
        assert!(log_content.contains("Hello, Ghost!"));
        assert!(log_content.contains("Error message"));
    }

    #[test]
    fn test_kill_process_force() {
        // Test that SIGKILL works even when SIGTERM is trapped

        let temp_dir = tempfile::tempdir().unwrap();
        let log_dir = temp_dir.path().to_path_buf();

        // Create a script that ignores SIGTERM
        let script_content = "#!/bin/sh\ntrap '' TERM\nwhile true; do sleep 1; done";
        let script_path = temp_dir.path().join("ignore_term.sh");
        std::fs::write(&script_path, script_content).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms).unwrap();
        }

        let command = vec![script_path.to_string_lossy().to_string()];
        let result = spawn_background_process(command, Some(log_dir));
        assert!(result.is_ok());

        let (process_info, mut child) = result.unwrap();
        let pid = process_info.pid;

        // Verify process is running
        assert!(process_exists(pid));

        // Try to kill with SIGTERM (should not work due to trap)
        let _ = kill_process(pid, false);
        thread::sleep(Duration::from_millis(200));

        // Force kill with SIGKILL - this should always work
        let kill_result = kill_process(pid, true);
        assert!(kill_result.is_ok());

        // Wait to reap the zombie
        let _ = child.wait();
        assert!(!process_exists(pid));
    }
}
