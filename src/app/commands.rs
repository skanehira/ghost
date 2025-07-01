use std::path::PathBuf;

use crate::app::{error::Result, process, storage};

/// Run a command in the background
pub fn run(command: Vec<String>, cwd: Option<PathBuf>, env: Vec<String>) -> Result<()> {
    if command.is_empty() {
        return Err("No command specified".into());
    }

    // Parse environment variables
    let mut env_vars = Vec::new();
    for env_str in env {
        if let Some((key, value)) = env_str.split_once('=') {
            env_vars.push((key.to_string(), value.to_string()));
        } else {
            return Err(
                format!("Invalid environment variable format: {env_str}. Use KEY=VALUE").into(),
            );
        }
    }

    // Initialize database
    let conn = storage::init_database()?;

    // Spawn the process
    let (process_info, child) = process::spawn_background_process(command.clone(), None)?;

    // Insert into database
    storage::insert_task(
        &conn,
        &process_info.id,
        process_info.pid,
        #[cfg(unix)]
        Some(process_info.pgid),
        #[cfg(not(unix))]
        None,
        &command,
        if env_vars.is_empty() {
            None
        } else {
            Some(&env_vars)
        },
        cwd.as_deref(),
        &process_info.log_path,
    )?;

    println!("Started background process:");
    println!("  Task ID: {}", process_info.id);
    println!("  PID: {}", process_info.pid);
    println!("  Log file: {}", process_info.log_path.display());

    // Drop child to avoid zombie (we're not waiting)
    std::mem::drop(child);

    Ok(())
}

/// List all background processes
pub fn list(status_filter: Option<String>) -> Result<()> {
    let conn = storage::init_database()?;
    let tasks = storage::get_tasks_with_process_check(&conn, status_filter.as_deref())?;

    if tasks.is_empty() {
        println!("No tasks found.");
        return Ok(());
    }

    // Print table header
    println!(
        "{:<8} {:<8} {:<10} {:<20} {:<30}",
        "Task ID", "PID", "Status", "Started", "Command"
    );
    println!("{}", "-".repeat(80));

    for task in tasks {
        let command: Vec<String> = serde_json::from_str(&task.command).unwrap_or_default();
        let command_str = command.join(" ");
        let command_display = if command_str.len() > 30 {
            format!("{}...", &command_str[..27])
        } else {
            command_str
        };

        let started = chrono::DateTime::from_timestamp(task.started_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        println!(
            "{:<8} {:<8} {:<10} {:<20} {:<30}",
            &task.id[..8], // Show first 8 chars of task ID
            task.pid,
            task.status,
            started,
            command_display
        );
    }

    Ok(())
}

/// Show logs for a process
pub fn log(task_id: &str, follow: bool) -> Result<()> {
    let conn = storage::init_database()?;
    let task = storage::get_task(&conn, task_id)?;

    let log_path = PathBuf::from(&task.log_path);
    if !log_path.exists() {
        return Err(format!("Log file not found: {}", task.log_path).into());
    }

    if follow {
        // Simple follow implementation (could be improved)
        println!("Following logs for task {task_id} (Ctrl+C to stop):");
        println!("Log file: {}", task.log_path);
        println!("{}", "-".repeat(40));

        // For now, just read the current content
        // A real implementation would use inotify or similar
        let content = std::fs::read_to_string(&log_path)?;
        print!("{content}");

        // TODO: Implement proper follow functionality
        println!("\n[Follow mode not fully implemented yet]");
    } else {
        let content = std::fs::read_to_string(&log_path)?;
        print!("{content}");
    }

    Ok(())
}

/// Stop a background process
pub fn stop(task_id: &str, force: bool) -> Result<()> {
    let conn = storage::init_database()?;
    let task = storage::get_task(&conn, task_id)?;

    if task.status != storage::TaskStatus::Running {
        return Err(format!("Task {} is not running (status: {})", task_id, task.status).into());
    }

    // Kill the process
    process::kill(task.pid, force)?;

    // Update status in database
    let status = if force {
        storage::TaskStatus::Killed
    } else {
        storage::TaskStatus::Exited
    };
    storage::update_task_status(&conn, task_id, status, None)?;

    println!("Process {} ({}) has been {}", task_id, task.pid, status);

    Ok(())
}

/// Check status of a background process
pub fn status(task_id: &str) -> Result<()> {
    let conn = storage::init_database()?;

    // This will update the status if the process is no longer running
    let task = storage::update_task_status_by_process_check(&conn, task_id)?;

    println!("Task: {}", task.id);
    println!("PID: {}", task.pid);
    println!("Status: {}", task.status);

    let command: Vec<String> = serde_json::from_str(&task.command).unwrap_or_default();
    println!("Command: {}", command.join(" "));

    let started = chrono::DateTime::from_timestamp(task.started_at, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    println!("Started: {started}");

    if let Some(finished_at) = task.finished_at {
        let finished = chrono::DateTime::from_timestamp(finished_at, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        println!("Finished: {finished}");
    }

    if let Some(exit_code) = task.exit_code {
        println!("Exit code: {exit_code}");
    }

    println!("Log file: {}", task.log_path);

    Ok(())
}

/// Kill a process by PID (legacy command)
pub fn kill(pid: u32) -> Result<()> {
    process::kill(pid, true)?;
    println!("Process {pid} killed successfully.");
    Ok(())
}
