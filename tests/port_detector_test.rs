use ghost::app::port_detector::detect_listening_ports;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
fn test_detect_listening_ports_with_server() {
    // Start a test HTTP server
    let mut server = Command::new("python3")
        .args(["-m", "http.server", "0"]) // Use port 0 for random port
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to start test server");

    let server_pid = server.id();

    // Give the server time to start
    thread::sleep(Duration::from_secs(2));

    // Test port detection
    let result = detect_listening_ports(server_pid);
    assert!(result.is_ok());

    let ports = result.unwrap();
    assert!(
        !ports.is_empty(),
        "Should detect at least one listening port"
    );

    // Verify port information
    let port = &ports[0];
    assert_eq!(port.protocol, "tcp");
    assert!(port.local_addr.contains(":"));
    assert_eq!(port.state, "LISTEN");

    // Clean up
    let _ = server.kill();
    let _ = server.wait();
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
