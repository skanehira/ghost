use clap::{Parser, Subcommand};
use std::path::PathBuf;

use ghost::core::{command, storage};

#[derive(Parser, Debug)]
#[command(name = "ghost")]
#[command(about = "A simple background process manager")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a command in the background
    Run {
        /// The command to run
        command: Vec<String>,

        /// Working directory for the command
        #[arg(short, long)]
        cwd: Option<PathBuf>,

        /// Environment variables (KEY=VALUE format)
        #[arg(short, long)]
        env: Vec<String>,
    },

    /// List all background processes
    List {
        /// Filter by status (running, exited, killed)
        #[arg(short, long)]
        status: Option<String>,
    },

    /// Show logs for a process
    Log {
        /// Task ID to show logs for
        task_id: String,

        /// Follow log output (like tail -f)
        #[arg(short, long)]
        follow: bool,
    },

    /// Stop a background process
    Stop {
        /// Task ID to stop
        task_id: String,

        /// Force kill the process (SIGKILL instead of SIGTERM)
        #[arg(short, long)]
        force: bool,
    },

    /// Check status of a background process
    Status {
        /// Task ID to check
        task_id: String,
    },

    /// Kill a process by PID (legacy command)
    Kill {
        /// Process ID to kill
        pid: u32,
    },
}

fn main() {
    if cfg!(windows) {
        eprintln!("ghost does not support Windows yet.");
        std::process::exit(1);
    }

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Run { command, cwd, env } => cmd_run(command, cwd, env),
        Commands::List { status } => cmd_list(status),
        Commands::Log { task_id, follow } => cmd_log(&task_id, follow),
        Commands::Stop { task_id, force } => cmd_stop(&task_id, force),
        Commands::Status { task_id } => cmd_status(&task_id),
        Commands::Kill { pid } => cmd_kill(pid),
    };

    if let Err(e) = result {
        eprintln!(r#"Error: {e}"#);
        std::process::exit(1);
    }
}

fn cmd_run(
    command: Vec<String>,
    cwd: Option<PathBuf>,
    env: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
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
    let (process_info, child) = command::spawn_background_process(command.clone(), None)?;

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

fn cmd_list(status_filter: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
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

fn cmd_log(task_id: &str, follow: bool) -> Result<(), Box<dyn std::error::Error>> {
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

fn cmd_stop(task_id: &str, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = storage::init_database()?;
    let task = storage::get_task(&conn, task_id)?;

    if task.status != "running" {
        return Err(format!("Task {} is not running (status: {})", task_id, task.status).into());
    }

    // Kill the process
    command::kill_process(task.pid, force)?;

    // Update status in database
    let status = if force { "killed" } else { "exited" };
    storage::update_task_status(&conn, task_id, status, None)?;

    println!("Process {} ({}) has been {}", task_id, task.pid, status);

    Ok(())
}

fn cmd_status(task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
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

fn cmd_kill(pid: u32) -> Result<(), Box<dyn std::error::Error>> {
    command::kill_process(pid, true)?;
    println!("Process {pid} killed successfully.");
    Ok(())
}
