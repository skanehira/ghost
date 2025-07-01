use std::path::PathBuf;

use crate::app::{config, display, error::Result, helpers, process, process_state, storage};
use rusqlite::Connection;

/// Run a command in the background
pub fn run(command: Vec<String>, cwd: Option<PathBuf>, env: Vec<String>) -> Result<()> {
    validate_command(&command)?;
    let env_vars = prepare_environment(&env)?;
    let conn = helpers::init_db_connection()?;
    let (process_info, child) = spawn_and_register_process(command, cwd, env_vars, &conn)?;
    finalize_process_launch(process_info, child);
    Ok(())
}

/// Validate the command input
fn validate_command(command: &[String]) -> Result<()> {
    if command.is_empty() {
        return Err("No command specified".into());
    }
    Ok(())
}

/// Prepare and parse environment variables
fn prepare_environment(env: &[String]) -> Result<Vec<(String, String)>> {
    config::env::parse_env_vars(env)
}

/// Spawn process and register it in the database
fn spawn_and_register_process(
    command: Vec<String>,
    cwd: Option<PathBuf>,
    env_vars: Vec<(String, String)>,
    conn: &Connection,
) -> Result<(process::ProcessInfo, std::process::Child)> {
    let (process_info, child) = process::spawn_background_process(command.clone(), None)?;

    storage::insert_task(
        conn,
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

    Ok((process_info, child))
}

/// Finalize the process launch with notification and cleanup
fn finalize_process_launch(process_info: process::ProcessInfo, child: std::process::Child) {
    display::print_process_started(&process_info.id, process_info.pid, &process_info.log_path);
    std::mem::drop(child);
}

/// List all background processes
pub fn list(status_filter: Option<String>) -> Result<()> {
    let conn = helpers::init_db_connection()?;
    let tasks = storage::get_tasks_with_process_check(&conn, status_filter.as_deref())?;
    display::print_task_list(&tasks);

    Ok(())
}

/// Show logs for a process
pub fn log(task_id: &str, follow: bool) -> Result<()> {
    let conn = helpers::init_db_connection()?;
    let task = helpers::get_task_by_id(&conn, task_id)?;

    let log_path = PathBuf::from(&task.log_path);
    let content = helpers::read_file_content(&log_path)?;

    if follow {
        display::print_log_follow_header(task_id, &task.log_path);
        helpers::print_file_content(&content);

        // TODO: Implement proper follow functionality
        println!("\n[Follow mode not fully implemented yet]");
    } else {
        helpers::print_file_content(&content);
    }

    Ok(())
}

/// Stop a background process
pub fn stop(task_id: &str, force: bool) -> Result<()> {
    let conn = helpers::init_db_connection()?;
    let task = helpers::get_task_by_id(&conn, task_id)?;

    helpers::validate_task_running(&task)?;

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
    let conn = helpers::init_db_connection()?;

    // This will update the status if the process is no longer running
    let task = helpers::get_task_with_status_update(&conn, task_id)?;
    display::print_task_details(&task);

    Ok(())
}

/// Kill a process by PID (legacy command)
pub fn kill(pid: u32) -> Result<()> {
    process::kill(pid, true)?;
    println!("Process {pid} killed successfully.");
    Ok(())
}
