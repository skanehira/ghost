use crate::app::storage::Task;

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

        println!(
            "{:<8} {:<8} {:<10} {:<20} {:<30}",
            truncate_string(&task.id, 8),
            task.pid,
            task.status,
            started,
            command_display
        );
    }
}

/// Print the table header for task list
fn print_table_header() {
    println!(
        "{:<8} {:<8} {:<10} {:<20} {:<30}",
        "Task ID", "PID", "Status", "Started", "Command"
    );
    println!("{}", "-".repeat(80));
}

/// Display detailed information about a single task
pub fn print_task_details(task: &Task) {
    println!("Task: {}", task.id);
    println!("PID: {}", task.pid);
    println!("Status: {}", task.status);
    println!("Command: {}", format_command_full(&task.command));

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

    println!("Log file: {}", task.log_path);
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
fn truncate_string(s: &str, max_length: usize) -> String {
    if s.len() > max_length {
        if max_length >= 3 {
            format!("{}...", &s[..max_length - 3])
        } else {
            s[..max_length].to_string()
        }
    } else {
        s.to_string()
    }
}
