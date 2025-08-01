/// Utilities for extracting port information from running processes
/// Extract port number from process using lsof (including child processes)
pub fn extract_port_from_process(pid: u32) -> String {
    // First, get all child processes of this PID
    let mut pids_to_check = vec![pid];

    // Get child processes using ps
    if let Ok(output) = std::process::Command::new("ps")
        .args(["-o", "pid,ppid", "-A"])
        .output()
    {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines().skip(1) {
                // Skip header
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let (Ok(child_pid), Ok(parent_pid)) =
                        (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                    {
                        if parent_pid == pid {
                            pids_to_check.push(child_pid);
                        }
                    }
                }
            }
        }
    }

    // Check each PID for listening ports
    for check_pid in pids_to_check {
        if let Some(port) = extract_port_from_single_process(check_pid) {
            return port;
        }
    }

    "-".to_string()
}

/// Extract port from a single process PID
fn extract_port_from_single_process(pid: u32) -> Option<String> {
    let output = std::process::Command::new("lsof")
        .args(["-p", &pid.to_string(), "-i", "-P", "-n"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let pid_str = pid.to_string();

    // Parse lsof output to find listening ports for the specific PID
    for line in stdout.lines() {
        if line.contains("LISTEN") && line.contains("TCP") && line.contains(&pid_str) {
            if let Some(port) = extract_port_from_lsof_line(line) {
                return Some(port.to_string());
            }
        }
    }

    None
}

/// Extract web server info from process ID using lsof (returns :port format for TUI compatibility)
pub fn extract_web_server_info(pid: u32) -> Option<String> {
    // First, get all child processes of this PID
    let mut pids_to_check = vec![pid];

    // Get child processes using ps
    if let Ok(output) = std::process::Command::new("ps")
        .args(["-o", "pid,ppid", "-A"])
        .output()
    {
        if let Ok(stdout) = String::from_utf8(output.stdout) {
            for line in stdout.lines().skip(1) {
                // Skip header
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let (Ok(child_pid), Ok(parent_pid)) =
                        (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                    {
                        if parent_pid == pid {
                            pids_to_check.push(child_pid);
                        }
                    }
                }
            }
        }
    }

    // Check each PID for listening ports
    for check_pid in pids_to_check {
        if let Some(port) = extract_port_from_single_process_for_web(check_pid) {
            return Some(format!(":{port}"));
        }
    }

    None
}

/// Extract port from a single process PID for web server info (returns port number only)
fn extract_port_from_single_process_for_web(pid: u32) -> Option<u16> {
    let output = std::process::Command::new("lsof")
        .args(["-p", &pid.to_string(), "-i", "-P", "-n"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let pid_str = pid.to_string();

    // Parse lsof output to find listening ports for the specific PID
    for line in stdout.lines() {
        if line.contains("LISTEN") && line.contains("TCP") && line.contains(&pid_str) {
            if let Some(port) = extract_port_from_lsof_line(line) {
                return Some(port);
            }
        }
    }

    None
}

/// Extract port number from lsof output line
fn extract_port_from_lsof_line(line: &str) -> Option<u16> {
    // Look for patterns like "*:3000", "localhost:3000", "[::1]:3000", "127.0.0.1:3000"
    let patterns = [
        r"\*:(\d+)",           // *:3000
        r"\blocalhost:(\d+)",  // localhost:3000
        r"\[::1\]:(\d+)",      // [::1]:3000 (IPv6 localhost)
        r"127\.0\.0\.1:(\d+)", // 127.0.0.1:3000
    ];

    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            if let Some(captures) = re.captures(line) {
                if let Some(port_str) = captures.get(1) {
                    if let Ok(port) = port_str.as_str().parse::<u16>() {
                        return Some(port);
                    }
                }
            }
        }
    }
    None
}
