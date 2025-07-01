use crate::app::storage::Task;

/// Display a list of tasks in a formatted table
pub fn print_task_list(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("No tasks found.");
        return;
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
}

/// Display detailed information about a single task
pub fn print_task_details(task: &Task) {
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
