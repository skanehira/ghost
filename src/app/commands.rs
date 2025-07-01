use std::path::PathBuf;

use crate::app::{config, display, error::Result, process, process_state, storage};

/// Run a command in the background
pub fn run(command: Vec<String>, cwd: Option<PathBuf>, env: Vec<String>) -> Result<()> {
    if command.is_empty() {
        return Err("No command specified".into());
    }

    // Parse environment variables
    let env_vars = config::env::parse_env_vars(&env)?;

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

    display::print_process_started(&process_info.id, process_info.pid, &process_info.log_path);

    // Drop child to avoid zombie (we're not waiting)
    std::mem::drop(child);

    Ok(())
}

/// List all background processes
pub fn list(status_filter: Option<String>) -> Result<()> {
    let conn = storage::init_database()?;
    let tasks = storage::get_tasks_with_process_check(&conn, status_filter.as_deref())?;
    display::print_task_list(&tasks);

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
        display::print_log_follow_header(task_id, &task.log_path);

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
    let status = process_state::determine_status_after_kill(force);
    storage::update_task_status(&conn, task_id, status, None)?;

    println!("Process {} ({}) has been {}", task_id, task.pid, status);

    Ok(())
}

/// Check status of a background process
pub fn status(task_id: &str) -> Result<()> {
    let conn = storage::init_database()?;

    // This will update the status if the process is no longer running
    let task = storage::update_task_status_by_process_check(&conn, task_id)?;
    display::print_task_details(&task);

    Ok(())
}

/// Kill a process by PID (legacy command)
pub fn kill(pid: u32) -> Result<()> {
    process::kill(pid, true)?;
    println!("Process {pid} killed successfully.");
    Ok(())
}
