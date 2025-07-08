use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use nix::unistd::setsid;
use std::fs::File;
use std::os::unix::process::CommandExt as _;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub id: String,
    pub pid: u32,
    pub pgid: i32,
    pub command: Vec<String>,
    pub log_path: PathBuf,
    pub env: Vec<(String, String)>,
}

use crate::app::error::{GhostError, Result};

/// Spawn a background process with logging
/// Returns both ProcessInfo and Child handle to allow proper cleanup
pub fn spawn_background_process(
    command: Vec<String>,
    cwd: Option<PathBuf>,
    log_dir: Option<PathBuf>,
) -> Result<(ProcessInfo, Child)> {
    spawn_background_process_with_env(command, cwd, log_dir, Vec::new())
}

/// Spawn a background process with logging and custom environment variables
/// Returns both ProcessInfo and Child handle to allow proper cleanup
pub fn spawn_background_process_with_env(
    command: Vec<String>,
    cwd: Option<PathBuf>,
    log_dir: Option<PathBuf>,
    custom_env: Vec<(String, String)>,
) -> Result<(ProcessInfo, Child)> {
    // Generate task ID and prepare paths
    let task_id = Uuid::new_v4().to_string();
    let log_dir = log_dir.unwrap_or_else(crate::app::config::get_log_dir);

    // Create log directory if it doesn't exist
    std::fs::create_dir_all(&log_dir)?;

    let log_path = log_dir.join(format!("{task_id}.log"));

    // Create log file
    let log_file = File::create(&log_path).map_err(|e| GhostError::LogFileCreation {
        path: log_path.to_string_lossy().to_string(),
        source: e,
    })?;

    // Setup command
    let mut cmd = Command::new(&command[0]);
    cmd.args(&command[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::from(log_file.try_clone()?))
        .stderr(Stdio::from(log_file));

    // Set current working directory if specified
    if let Some(ref cwd) = cwd {
        cmd.current_dir(cwd);
    }

    // Collect all environment variables (inherited + custom)
    let mut all_env: Vec<(String, String)> = std::env::vars().collect();

    // Add custom environment variables
    for (key, value) in &custom_env {
        cmd.env(key, value);
        // Update or add to all_env
        if let Some(pos) = all_env.iter().position(|(k, _)| k == key) {
            all_env[pos] = (key.clone(), value.clone());
        } else {
            all_env.push((key.clone(), value.clone()));
        }
    }

    unsafe {
        cmd.pre_exec(|| setsid().map(|_| ()).map_err(std::io::Error::other));
    }

    // Spawn the process
    let child = cmd.spawn().map_err(|e| GhostError::ProcessSpawn {
        message: format!("Failed to spawn process: {e}"),
    })?;

    let pid = child.id();

    // The process group ID should be the same as PID after setsid()
    let pgid = pid as i32;

    let info = ProcessInfo {
        id: task_id,
        pid,
        pgid,
        command,
        log_path,
        env: all_env,
    };

    Ok((info, child))
}

/// Check if a process is still running
pub fn exists(pid: u32) -> bool {
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
pub fn kill(pid: u32, force: bool) -> Result<()> {
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
pub fn kill_group(pgid: i32, force: bool) -> Result<()> {
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
        let result = spawn_background_process(command.clone(), None, Some(log_dir.clone()));

        assert!(result.is_ok());
        let (process_info, mut child) = result.unwrap();

        // Check process info
        assert!(!process_info.id.is_empty());
        assert!(process_info.pid > 0);
        assert_eq!(process_info.command, command);
        assert!(!process_info.env.is_empty()); // Should have inherited environment

        // Check log file exists
        assert!(process_info.log_path.exists());

        // Check process is running
        assert!(exists(process_info.pid));

        // Wait a bit and check again
        thread::sleep(Duration::from_millis(100));
        assert!(exists(process_info.pid));

        // Kill the process
        let _ = kill(process_info.pid, true);

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

        let result = spawn_background_process(command, None, Some(log_dir));
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
    fn test_spawn_with_cwd() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_dir = temp_dir.path().to_path_buf();
        let test_cwd = temp_dir.path().join("test_cwd");

        // Create test directory and file
        std::fs::create_dir_all(&test_cwd).unwrap();
        let test_file = test_cwd.join("test_file.txt");
        std::fs::write(&test_file, "test content").unwrap();

        // Spawn command that reads the file in the specified cwd
        let command = vec!["cat".to_string(), "test_file.txt".to_string()];
        let result = spawn_background_process(command, Some(test_cwd), Some(log_dir));

        assert!(result.is_ok());
        let (process_info, mut child) = result.unwrap();

        // Wait for process to complete
        let _ = child.wait();

        // Check log content to verify cwd was used
        let log_content = std::fs::read_to_string(&process_info.log_path).unwrap();
        assert!(log_content.contains("test content"));
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

        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms).unwrap();

        let command = vec![script_path.to_string_lossy().to_string()];
        let result = spawn_background_process(command, None, Some(log_dir));
        assert!(result.is_ok());

        let (process_info, mut child) = result.unwrap();
        let pid = process_info.pid;

        // Verify process is running
        assert!(exists(pid));

        // Try to kill with SIGTERM (should not work due to trap)
        let _ = kill(pid, false);
        thread::sleep(Duration::from_millis(200));

        // Force kill with SIGKILL - this should always work
        let kill_result = kill(pid, true);
        assert!(kill_result.is_ok());

        // Wait to reap the zombie
        let _ = child.wait();
        assert!(!exists(pid));
    }

    #[test]
    fn test_spawn_with_custom_env() {
        let temp_dir = tempfile::tempdir().unwrap();
        let log_dir = temp_dir.path().to_path_buf();

        // Spawn command with custom environment variable
        let command = vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo $TEST_CUSTOM_VAR".to_string(),
        ];
        let custom_env = vec![("TEST_CUSTOM_VAR".to_string(), "Hello Ghost!".to_string())];

        let result = spawn_background_process_with_env(
            command.clone(),
            None,
            Some(log_dir),
            custom_env.clone(),
        );

        assert!(result.is_ok());
        let (process_info, mut child) = result.unwrap();

        // Check that custom env var is in the process info
        let has_custom_var = process_info
            .env
            .iter()
            .any(|(k, v)| k == "TEST_CUSTOM_VAR" && v == "Hello Ghost!");
        assert!(has_custom_var);

        // Wait for process to complete
        let _ = child.wait();

        // Check log content to verify env var was used
        let log_content = std::fs::read_to_string(&process_info.log_path).unwrap();
        assert!(log_content.contains("Hello Ghost!"));
    }
}
