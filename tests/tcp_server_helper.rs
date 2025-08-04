use std::net::TcpListener;
use std::thread;
use std::time::Duration;

fn main() {
    // Get port from command line args, default to 0 (random port)
    let port = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);

    let listener =
        TcpListener::bind(format!("127.0.0.1:{port}")).expect("Failed to bind TCP listener");

    let addr = listener.local_addr().unwrap();
    println!("Listening on {addr}");

    // Keep the server running for 30 seconds
    thread::sleep(Duration::from_secs(30));
}
