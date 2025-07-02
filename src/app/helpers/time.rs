use std::time::{SystemTime, UNIX_EPOCH};

/// Get current Unix timestamp in seconds
pub fn now_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
