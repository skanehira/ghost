use std::io::{self, Write};
use std::process::{Command, Stdio};

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Zombie Process Demo ===");

    // Case 1: spawn + drop (ゾンビになる)
    println!("\n1. Spawning process and dropping Child handle...");
    let child = Command::new("sleep")
        .arg("30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let pid = child.id();
    println!("Process PID: {pid}");

    // childをドロップ（wait()は呼ばれない）
    drop(child);

    println!("Child handle dropped. Press Enter to kill process...");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    // プロセスをkill
    signal::kill(Pid::from_raw(pid as i32), Signal::SIGKILL)?;
    println!("Process killed with SIGKILL");

    println!("Check zombie with: ps -ef | grep {pid} | grep defunct");
    println!("Press Enter to continue...");
    io::stdin().read_line(&mut input)?;

    // Case 2: spawn + wait (ゾンビにならない)
    println!("\n2. Spawning process and calling wait()...");
    let mut child = Command::new("sleep")
        .arg("30")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let pid2 = child.id();
    println!("Process PID: {pid2}");

    println!("Killing process...");
    signal::kill(Pid::from_raw(pid2 as i32), Signal::SIGKILL)?;

    println!("Calling wait()...");
    let status = child.wait()?;
    println!("Process reaped with status: {status:?}");

    println!("Check if zombie exists: ps -ef | grep {pid2} | grep defunct",);
    println!("(Should not find any zombie)");

    println!("\nDemo complete!");
    Ok(())
}
