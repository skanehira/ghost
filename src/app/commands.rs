use std::path::PathBuf;

use crate::app::{config, display, error::Result, helpers, process, process_state, storage};
use rusqlite::Connection;

/// Run a command in the background
pub fn spawn(command: Vec<String>, cwd: Option<PathBuf>, env: Vec<String>) -> Result<()> {
    if command.is_empty() {
        return Err("No command specified".into());
    }
    let env_vars = config::env::parse_env_vars(&env)?;
    let conn = storage::init_database()?;
    let (process_info, child) = spawn_and_register_process(command, cwd, env_vars, &conn)?;
    finalize_process_launch(process_info, child);
    Ok(())
}

/// Spawn process and register it in the database
fn spawn_and_register_process(
    command: Vec<String>,
    cwd: Option<PathBuf>,
    env_vars: Vec<(String, String)>,
    conn: &Connection,
) -> Result<(process::ProcessInfo, std::process::Child)> {
    let (process_info, child) =
        process::spawn_background_process(command.clone(), cwd.clone(), None)?;

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
    let conn = storage::init_database()?;
    let tasks = storage::get_tasks_with_process_check(&conn, status_filter.as_deref())?;
    display::print_task_list(&tasks);

    Ok(())
}

/// Show logs for a process
pub async fn log(task_id: &str, follow: bool) -> Result<()> {
    let conn = storage::init_database()?;
    let task = storage::get_task(&conn, task_id)?;

    let log_path = PathBuf::from(&task.log_path);
    let content = helpers::read_file_content(&log_path)?;

    if follow {
        display::print_log_follow_header(task_id, &task.log_path);
        helpers::follow_log_file(&log_path).await?;
    } else {
        print!("{content}");
    }

    Ok(())
}

/// Stop a background process
pub fn stop(task_id: &str, force: bool) -> Result<()> {
    let conn = storage::init_database()?;
    let task = storage::get_task(&conn, task_id)?;

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

/// Clean up old finished tasks
pub fn cleanup(days: u64, status: Option<String>, dry_run: bool, all: bool) -> Result<()> {
    let conn = storage::init_database()?;

    // Parse status filter
    let status_filter = parse_status_filter(status.as_deref())?;

    // Determine days filter - None if --all is specified
    let days_filter = if all { None } else { Some(days) };

    if dry_run {
        // Show what would be deleted
        let candidates = storage::get_cleanup_candidates(&conn, days_filter, &status_filter)?;

        if candidates.is_empty() {
            println!("No tasks found matching cleanup criteria.");
            return Ok(());
        }

        println!(
            "The following {} task(s) would be deleted:",
            candidates.len()
        );
        display::print_task_list(&candidates);

        if all {
            println!(
                "\nNote: --all flag specified, all finished tasks would be deleted regardless of age."
            );
        } else {
            println!("\nNote: Only tasks older than {days} days would be deleted.");
        }
    } else {
        // Actually delete tasks
        let deleted_count = storage::cleanup_tasks_by_criteria(&conn, days_filter, &status_filter)?;

        if deleted_count == 0 {
            println!("No tasks found matching cleanup criteria.");
        } else {
            println!("Successfully deleted {deleted_count} task(s).");

            if all {
                println!("Deleted all finished tasks regardless of age.");
            } else {
                println!(
                    "Deleted tasks older than {} days with status: {}.",
                    days,
                    format_status_list(&status_filter)
                );
            }
        }
    }

    Ok(())
}

/// Parse status filter string into TaskStatus enum list
fn parse_status_filter(status: Option<&str>) -> Result<Vec<storage::TaskStatus>> {
    match status {
        Some("all") => {
            // All statuses except running (don't delete running tasks)
            Ok(vec![
                storage::TaskStatus::Exited,
                storage::TaskStatus::Killed,
                storage::TaskStatus::Unknown,
            ])
        }
        Some(status_str) => {
            let statuses: Result<Vec<_>> = status_str
                .split(',')
                .map(|s| s.trim())
                .map(|s| match s {
                    "exited" => Ok(storage::TaskStatus::Exited),
                    "killed" => Ok(storage::TaskStatus::Killed),
                    "unknown" => Ok(storage::TaskStatus::Unknown),
                    "running" => Err("Cannot cleanup running tasks".into()),
                    _ => Err(format!(
                        "Invalid status: {s}. Valid options: exited, killed, unknown, all"
                    )
                    .into()),
                })
                .collect();
            statuses
        }
        None => {
            // Default: exited and killed only
            Ok(vec![
                storage::TaskStatus::Exited,
                storage::TaskStatus::Killed,
            ])
        }
    }
}

/// Format status list for display
fn format_status_list(statuses: &[storage::TaskStatus]) -> String {
    statuses
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}
