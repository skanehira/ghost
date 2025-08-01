use crate::app::helpers::extract_port_from_process;
use crate::app::storage::{task_status::TaskStatus, Task};

/// Display a list of tasks in a formatted table
pub fn print_task_list(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("No tasks found.");
        return;
    }

    print_table_header();

    for task in tasks {
        let command_display = format_command_truncated(&task.command, 30);
        let started = format_timestamp(task.started_at, "%Y-%m-%d %H:%M");
        let cwd_display = task.cwd.as_deref().unwrap_or("-");
        let port_display = if task.status == TaskStatus::Running {
            extract_port_from_process(task.pid)
        } else {
            "-".to_string()
        };

        println!(
            "{:<36} {:<8} {:<10} {:<20} {:<6} {:<30} {}",
            &task.id,
            task.pid,
            task.status.as_str(),
            started,
            port_display,
            command_display,
            cwd_display
        );
    }
}

/// Print the table header for task list
fn print_table_header() {
    println!(
        "{:<36} {:<8} {:<10} {:<20} {:<6} {:<30} Directory",
        "Task ID", "PID", "Status", "Started", "Port", "Command"
    );
    println!("{}", "-".repeat(140));
}

/// Display detailed information about a single task
pub fn print_task_details(task: &Task) {
    let task_id = &task.id;
    println!("Task: {task_id}");
    let pid = task.pid;
    println!("PID: {pid}");
    let status = &task.status;
    println!("Status: {status}");
    let command = format_command_full(&task.command);
    println!("Command: {command}");

    if let Some(ref cwd) = task.cwd {
        println!("Working directory: {cwd}");
    }

    println!(
        "Started: {}",
        format_timestamp(task.started_at, "%Y-%m-%d %H:%M:%S")
    );

    if let Some(finished_at) = task.finished_at {
        println!(
            "Finished: {}",
            format_timestamp(finished_at, "%Y-%m-%d %H:%M:%S")
        );
    }

    if let Some(exit_code) = task.exit_code {
        println!("Exit code: {exit_code}");
    }

    let log_path = &task.log_path;
    println!("Log file: {log_path}");
}

/// Display information about a started process
pub fn print_process_started(task_id: &str, pid: u32, log_path: &std::path::Path) {
    println!("Started background process:");
    println!("  Task ID: {task_id}");
    println!("  PID: {pid}");
    println!("  Log file: {}", log_path.display());
}

/// Display log follow header
pub fn print_log_follow_header(task_id: &str, log_path: &str) {
    println!("Following logs for task {task_id} (Ctrl+C to stop):");
    println!("Log file: {log_path}");
    println!("{}", "-".repeat(40));
}

// Helper functions for formatting

/// Format a command JSON string for display with truncation
fn format_command_truncated(command_json: &str, max_length: usize) -> String {
    let command_str = format_command_full(command_json);
    truncate_string(&command_str, max_length)
}

/// Format a command JSON string for full display
fn format_command_full(command_json: &str) -> String {
    let command: Vec<String> = serde_json::from_str(command_json).unwrap_or_default();
    command.join(" ")
}

/// Format a timestamp to a human-readable string
fn format_timestamp(timestamp: i64, format_str: &str) -> String {
    chrono::DateTime::from_timestamp(timestamp, 0)
        .map(|dt| dt.format(format_str).to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

/// Truncate a string to the specified length with ellipsis
/// This function respects UTF-8 character boundaries to prevent panics with multibyte characters
fn truncate_string(s: &str, max_length: usize) -> String {
    if s.chars().count() > max_length {
        if max_length >= 3 {
            let truncated: String = s.chars().take(max_length - 3).collect();
            format!("{truncated}...")
        } else {
            s.chars().take(max_length).collect()
        }
    } else {
        s.to_string()
    }
}
