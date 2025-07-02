use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader};
use tokio::sync::mpsc;

use crate::app::{error, error::Result};

/// Follow a log file and print new lines as they appear (tail -f behavior)
pub async fn follow_log_file(file_path: &PathBuf) -> Result<()> {
    use notify::{Config, PollWatcher, RecursiveMode, Watcher};
    use std::io::SeekFrom;

    if !tokio::fs::try_exists(file_path).await? {
        return Err(error::GhostError::InvalidArgument {
            message: format!("File not found: {path}", path = file_path.display()),
        });
    }

    // Read and print existing content first
    let mut file = File::open(file_path).await?;
    let mut reader = BufReader::new(&mut file);
    let mut line = String::new();

    while reader.read_line(&mut line).await? > 0 {
        print!("{line}");
        line.clear();
    }

    // Get current file position
    let mut last_position = file.stream_position().await?;

    // Set up file system watcher
    let (tx, mut rx) = mpsc::channel(100);
    let mut watcher = PollWatcher::new(
        move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                let _ = tx.blocking_send(event);
            }
        },
        Config::default().with_poll_interval(Duration::from_millis(200)),
    )
    .map_err(|e| error::GhostError::FileWatch {
        message: format!("Failed to create file watcher: {e}"),
    })?;

    // Watch the file for changes
    watcher
        .watch(file_path, RecursiveMode::NonRecursive)
        .map_err(|e| error::GhostError::FileWatch {
            message: format!("Failed to watch file: {e}"),
        })?;

    // Main event loop
    loop {
        tokio::select! {
            Some(event) = rx.recv() => {
                // File was modified, read new content
                if event.kind.is_modify() {
                    let metadata = tokio::fs::metadata(file_path).await?;
                    let current_size = metadata.len();

                    if current_size > last_position {
                        // File has grown, read new lines
                        let mut file = File::open(file_path).await?;
                        file.seek(SeekFrom::Start(last_position)).await?;
                        let mut reader = BufReader::new(file);
                        let mut line = String::new();

                        while reader.read_line(&mut line).await? > 0 {
                            print!("{line}");
                            use std::io::Write;
                            std::io::stdout().flush().unwrap_or(());
                            line.clear();
                        }

                        last_position = current_size;
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                // Ctrl+C was pressed, break the loop
                println!("\nLog following stopped.");
                break;
            }
        }
    }
    Ok(())
}
