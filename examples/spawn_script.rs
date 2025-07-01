use ghost::app::process;
use std::io::{self, Write};
use std::path::PathBuf;

fn print_status(pid: u32) {
    let running = process::exists(pid);

    println!(
        "Process is currently {}",
        if running { "running" } else { "not running" }
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = PathBuf::from("./ghost-demo-logs");
    std::fs::create_dir_all(&log_dir)?;

    println!("Spawning hello_loop.sh in background...");

    let command = vec!["./scripts/hello_loop.sh".to_string()];
    let (process_info, mut child) =
        process::spawn_background_process(command, Some(log_dir.clone()))?;

    println!(
        "Process started: ID={}, PID={}",
        process_info.id, process_info.pid
    );
    println!("Log file: {}", process_info.log_path.display());

    print_status(process_info.pid);

    println!("\nPress Enter to kill the process...");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    process::kill(process_info.pid, true)?;

    // Wait to reap the zombie
    let _ = child.wait();

    io::stdin().read_line(&mut input)?;

    print_status(process_info.pid);

    Ok(())
}
