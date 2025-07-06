use std::path::PathBuf;

use crate::app::{config, display, error, error::Result, helpers, process, storage};
use rusqlite::Connection;

/// Run a command in the background
pub fn spawn(command: Vec<String>, cwd: Option<PathBuf>, env: Vec<String>) -> Result<()> {
    if command.is_empty() {
        return Err(error::GhostError::InvalidArgument {
            message: "No command specified".to_string(),
        });
    }
    let env_vars = config::env::parse_env_vars(&env)?;
    let conn = storage::init_database()?;
    let (process_info, _) = spawn_and_register_process(command, cwd, env_vars, &conn)?;
    display::print_process_started(&process_info.id, process_info.pid, &process_info.log_path);
    Ok(())
}

/// Spawn process and register it in the database
fn spawn_and_register_process(
    command: Vec<String>,
    cwd: Option<PathBuf>,
    env_vars: Vec<(String, String)>,
    conn: &Connection,
) -> Result<(process::ProcessInfo, std::process::Child)> {
    // If no cwd is specified, use the current directory
    let effective_cwd = match cwd {
        Some(path) => Some(path),
        None => std::env::current_dir().ok(),
    };

    let (process_info, child) =
        process::spawn_background_process(command.clone(), effective_cwd.clone(), None)?;

    storage::insert_task(
        conn,
        &process_info.id,
        process_info.pid,
        Some(process_info.pgid),
        &command,
        if env_vars.is_empty() {
            None
        } else {
            Some(&env_vars)
        },
        effective_cwd.as_deref(),
        &process_info.log_path,
    )?;

    Ok((process_info, child))
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
    let content =
        std::fs::read_to_string(&log_path).map_err(|e| error::GhostError::InvalidArgument {
            message: format!("Failed to read log file: {e}"),
        })?;

    if follow {
        display::print_log_follow_header(task_id, &task.log_path);
        helpers::follow_log_file(&log_path).await?;
    } else {
        print!("{content}");
    }

    Ok(())
}

/// Stop a background process
pub fn stop(task_id: &str, force: bool, show_output: bool) -> Result<()> {
    let conn = storage::init_database()?;
    let task = storage::get_task(&conn, task_id)?;

    helpers::validate_task_running(&task)?;

    // Kill the process group if available, otherwise kill individual process
    if let Some(pgid) = task.pgid {
        process::kill_group(pgid, force)?;
    } else {
        process::kill(task.pid, force)?;
    }

    // Update status in database
    let status = if force {
        storage::TaskStatus::Killed
    } else {
        storage::TaskStatus::Exited
    };
    storage::update_task_status(&conn, task_id, status, None)?;

    if show_output {
        let pid = task.pid;
        println!("Process {task_id} ({pid}) has been {status}");
    }

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
                    "running" => Err(error::GhostError::InvalidArgument {
                        message: "Cannot cleanup running tasks".to_string(),
                    }),
                    _ => Err(error::GhostError::InvalidArgument {
                        message: format!(
                            "Invalid status: {s}. Valid options: exited, killed, unknown, all"
                        ),
                    }),
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

/// Start TUI mode
pub async fn tui() -> Result<()> {
    use crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use futures::StreamExt;
    use ratatui::{Terminal, backend::CrosstermBackend};
    use std::io;
    use tokio::time::{Duration, interval};

    use crate::app::tui::app::TuiApp;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = TuiApp::new()?;
    app.refresh_tasks()?;

    // Setup refresh interval and event stream
    let mut refresh_interval = interval(Duration::from_secs(1));
    let mut event_stream = EventStream::new();

    let result = loop {
        // Draw the UI
        terminal.draw(|f| app.render(f))?;

        // Handle input and refresh
        tokio::select! {
            // Handle keyboard events from async stream
            Some(event_result) = event_stream.next() => {
                match event_result {
                    Ok(Event::Key(key)) => {
                        if let Err(e) = app.handle_key(key) {
                            break Err(e);
                        }
                        if app.should_quit() {
                            break Ok(());
                        }
                    }
                    Err(e) => {
                        break Err(error::GhostError::Io { source: e });
                    }
                    _ => {} // Ignore other events (Mouse, Resize, etc.)
                }
            }

            // Refresh tasks periodically
            _ = refresh_interval.tick() => {
                if let Err(e) = app.refresh_tasks() {
                    break Err(e);
                }
            }
        }
    };

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}
