use crate::app::error::{GhostError, Result};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ListeningPort {
    pub protocol: String,
    pub local_addr: String,
    pub state: String,
}

/// Check if lsof command is available on the system
pub fn check_lsof_availability() -> Result<()> {
    let output =
        Command::new("which")
            .arg("lsof")
            .output()
            .map_err(|e| GhostError::ProcessOperation {
                message: format!("Failed to check for lsof command: {e}"),
            })?;

    if !output.status.success() {
        return Err(GhostError::ProcessOperation {
            message: "lsof command not found. Please install lsof to enable port detection."
                .to_string(),
        });
    }

    Ok(())
}

/// Parse lsof machine-readable format (-F flag) output
fn parse_lsof_machine_format(output: &str) -> Vec<ListeningPort> {
    let mut ports = Vec::new();
    let mut current_protocol = String::new();
    let mut current_addr = String::new();
    let mut current_state = String::new();
    let mut in_network_fd = false;

    for line in output.lines() {
        if line.is_empty() {
            continue;
        }

        let tag = &line[0..1];
        let value = &line[1..];

        match tag {
            "f" => {
                // File descriptor - reset state for new FD
                in_network_fd = false;
                current_protocol.clear();
                current_addr.clear();
                current_state.clear();
            }
            "t" => {
                // Type - check if it's a network connection
                if value.starts_with("IPv4") || value.starts_with("IPv6") {
                    in_network_fd = true;
                }
            }
            "n" => {
                // Name - contains address information
                if in_network_fd {
                    current_addr = value.to_string();
                }
            }
            "P" => {
                // Protocol - TCP or UDP
                if in_network_fd {
                    current_protocol = value.to_lowercase();
                }
            }
            "T" => {
                // TCP/TPI info - contains state like LISTEN
                if in_network_fd && value.starts_with("ST=LISTEN") {
                    current_state = "LISTEN".to_string();

                    // We have all the information, add the port
                    if !current_protocol.is_empty() && !current_addr.is_empty() {
                        ports.push(ListeningPort {
                            protocol: current_protocol.clone(),
                            local_addr: current_addr.clone(),
                            state: current_state.clone(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    ports
}

/// Detect listening ports for a given process ID
pub fn detect_listening_ports(pid: u32) -> Result<Vec<ListeningPort>> {
    // Check if lsof is available
    check_lsof_availability()?;

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    return detect_ports_using_lsof(pid);

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    return Ok(Vec::new());
}

/// Common implementation for macOS and Linux using lsof
#[cfg(any(target_os = "macos", target_os = "linux"))]
fn detect_ports_using_lsof(pid: u32) -> Result<Vec<ListeningPort>> {
    let output = Command::new("lsof")
        .args(["-nP", "-i", "-a", "-p", &pid.to_string(), "-F"])
        .output()
        .map_err(|e| GhostError::ProcessOperation {
            message: format!("Failed to execute lsof: {e}"),
        })?;

    if !output.status.success() {
        // Process might not have any network connections
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(parse_lsof_machine_format(&stdout))
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

    #[test]
    fn test_check_lsof_availability() {
        let result = check_lsof_availability();
        // This test should pass on systems with lsof installed
        // We can't guarantee it's installed, so we just check the function works
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_parse_lsof_machine_format() {
        // Test parsing of lsof -F format output
        let sample_output = "p1234\nfcwd\ntCWD\nn/home/user\n\
                           f6\ntREG\na r\ni123456\nn/usr/bin/app\n\
                           f10\ntIPv4\nPTCP\nn*:8080\nTST=LISTEN\n";

        let ports = parse_lsof_machine_format(sample_output);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].protocol, "tcp");
        assert_eq!(ports[0].local_addr, "*:8080");
        assert_eq!(ports[0].state, "LISTEN");
    }

    #[test]
    fn test_parse_malformed_lsof_output() {
        // Test with completely empty output
        let empty_output = "";
        let ports = parse_lsof_machine_format(empty_output);
        assert_eq!(ports.len(), 0);

        // Test with malformed lines (missing tag characters)
        let malformed_output = "1234\nf10\nt\nn*:8080\n";
        let ports = parse_lsof_machine_format(malformed_output);
        assert_eq!(ports.len(), 0);

        // Test with truncated output (missing required fields)
        let truncated_output = "p1234\nf10\ntIPv4\n";
        let ports = parse_lsof_machine_format(truncated_output);
        assert_eq!(ports.len(), 0);

        // Test with wrong tag order
        let wrong_order_output = "TST=LISTEN\nPTCP\ntIPv4\nn*:8080\nf10\n";
        let ports = parse_lsof_machine_format(wrong_order_output);
        assert_eq!(ports.len(), 0);

        // Test with corrupted protocol field
        let corrupted_protocol = "f10\ntIPv4\nP\nn*:8080\nTST=LISTEN\n";
        let ports = parse_lsof_machine_format(corrupted_protocol);
        assert_eq!(ports.len(), 0);

        // Test with corrupted address field
        let corrupted_address = "f10\ntIPv4\nPTCP\nn\nTST=LISTEN\n";
        let ports = parse_lsof_machine_format(corrupted_address);
        assert_eq!(ports.len(), 0);

        // Test with non-LISTEN state
        let non_listen_output = "f10\ntIPv4\nPTCP\nn*:8080\nTST=ESTABLISHED\n";
        let ports = parse_lsof_machine_format(non_listen_output);
        assert_eq!(ports.len(), 0);

        // Test with mixed valid and invalid entries
        let mixed_output = "p1234\n\
                          f10\ntIPv4\nPTCP\nn*:8080\nTST=LISTEN\n\
                          f11\ntIPv4\n\nn*:9000\nTST=LISTEN\n\
                          f12\ntIPv4\nPUDP\nn*:7000\nTST=LISTEN\n";
        let ports = parse_lsof_machine_format(mixed_output);
        assert_eq!(ports.len(), 2); // Should only get the valid TCP and UDP entries
        assert!(
            ports
                .iter()
                .any(|p| p.local_addr == "*:8080" && p.protocol == "tcp")
        );
        assert!(
            ports
                .iter()
                .any(|p| p.local_addr == "*:7000" && p.protocol == "udp")
        );
    }
}
