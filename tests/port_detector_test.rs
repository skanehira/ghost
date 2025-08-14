use ghost::app::port_detector::detect_listening_ports;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
fn test_detect_listening_ports_with_server() {
    // Compile the TCP server helper
    let output = Command::new("rustc")
        .args([
            "tests/tcp_server_helper.rs",
            "-o",
            "target/tcp_server_helper",
        ])
        .output()
        .expect("Failed to compile TCP server helper");

    if !output.status.success() {
        panic!(
            "Failed to compile TCP server helper: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Start the TCP server
    let mut child = Command::new("target/tcp_server_helper")
        .spawn()
        .expect("Failed to start TCP server");

    let server_pid = child.id();

    // Give the server more time to start and establish listening state
    thread::sleep(Duration::from_millis(1000));

    // Test port detection
    let result = detect_listening_ports(server_pid);
    assert!(result.is_ok());

    let ports = result.unwrap();
    assert!(
        !ports.is_empty(),
        "Should detect at least one listening port for PID {server_pid}"
    );

    // Verify port information
    let port = &ports[0];
    assert_eq!(port.protocol, "tcp");
    assert!(port.local_addr.contains(":"));
    assert_eq!(port.state, "LISTEN");

    // Clean up
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn test_detect_listening_ports_no_ports() {
    // Use a simple command that doesn't listen on any ports
    let mut sleep_process = Command::new("sleep")
        .arg("5")
        .spawn()
        .expect("Failed to start sleep process");

    let sleep_pid = sleep_process.id();

    // Test port detection
    let result = detect_listening_ports(sleep_pid);
    assert!(result.is_ok());

    let ports = result.unwrap();
    assert!(
        ports.is_empty(),
        "Should not detect any listening ports for sleep command"
    );

    // Clean up
    let _ = sleep_process.kill();
    let _ = sleep_process.wait();
}

#[test]
fn test_detect_listening_ports_invalid_pid() {
    // Use a PID that doesn't exist
    let result = detect_listening_ports(99999);
    assert!(result.is_ok());

    let ports = result.unwrap();
    assert!(
        ports.is_empty(),
        "Should return empty vec for non-existent PID"
    );
}
