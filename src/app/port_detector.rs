use crate::app::error::{GhostError, Result};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ListeningPort {
    pub protocol: String,
    pub local_addr: String,
    pub state: String,
}

/// Detect listening ports for a given process ID
pub fn detect_listening_ports(pid: u32) -> Result<Vec<ListeningPort>> {
    #[cfg(target_os = "macos")]
    return detect_ports_macos(pid);

    #[cfg(target_os = "linux")]
    return detect_ports_linux(pid);

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return Ok(Vec::new());
}

#[cfg(target_os = "macos")]
fn detect_ports_macos(pid: u32) -> Result<Vec<ListeningPort>> {
    let output = Command::new("lsof")
        .args(["-nP", "-i", "-a", "-p", &pid.to_string()])
        .output()
        .map_err(|e| GhostError::ProcessOperation {
            message: format!("Failed to execute lsof: {e}"),
        })?;

    if !output.status.success() {
        // Process might not have any network connections
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ports = Vec::new();

    for line in stdout.lines().skip(1) {
        // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        // Check if it's a LISTEN state
        if !parts[parts.len() - 1].contains("LISTEN") {
            continue;
        }

        // Extract protocol (TCP/UDP)
        let protocol = parts[7].to_lowercase();

        // Extract local address
        // lsof format: host:port or [::]:port
        let name_field = parts[8];
        if let Some(local_addr) = name_field.split("->").next() {
            ports.push(ListeningPort {
                protocol,
                local_addr: local_addr.to_string(),
                state: "LISTEN".to_string(),
            });
        }
    }

    Ok(ports)
}

#[cfg(target_os = "linux")]
fn detect_ports_linux(pid: u32) -> Result<Vec<ListeningPort>> {
    let output = Command::new("lsof")
        .args(["-nP", "-i", "-a", "-p", &pid.to_string()])
        .output()
        .map_err(|e| GhostError::ProcessOperation {
            message: format!("Failed to execute lsof: {e}"),
        })?;

    if !output.status.success() {
        // Process might not have any network connections
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ports = Vec::new();

    for line in stdout.lines().skip(1) {
        // Skip header
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 9 {
            continue;
        }

        // Check if it's a LISTEN state - last column contains state
        if !parts[parts.len() - 1].contains("LISTEN") {
            continue;
        }

        // Extract protocol (TCP/UDP) - column 7 (0-indexed)
        let protocol = parts[7].to_lowercase();

        // Extract local address - column 8 (0-indexed)
        // lsof format on Linux: *:port or IP:port
        let name_field = parts[8];
        if let Some(local_addr) = name_field.split("->").next() {
            ports.push(ListeningPort {
                protocol,
                local_addr: local_addr.to_string(),
                state: "LISTEN".to_string(),
            });
        }
    }

    Ok(ports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_listening_ports() {
        // This test requires a running process with known ports
        // In a real test, we would start a test server
        let ports = detect_listening_ports(std::process::id());
        assert!(ports.is_ok());
    }
}
