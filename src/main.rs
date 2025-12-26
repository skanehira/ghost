use clap::{Parser, Subcommand};
use rusqlite::Connection;
use std::path::PathBuf;

use ghost::app::{commands, config, error::Result, logging, storage};

#[derive(Parser, Debug)]
#[command(name = "ghost")]
#[command(about = "A simple background process manager")]
#[command(
    long_about = "A simple background process manager.\n\nRun without arguments to start the interactive TUI mode."
)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run one or more commands in the background
    ///
    /// Single command: ghost run sleep 10
    /// Multiple commands: ghost run "sleep 10" "echo hello"
    Run {
        /// Commands to run. For multiple commands, quote each command.
        /// Example: ghost run "sleep 10" "echo hello"
        #[arg(required = true)]
        commands: Vec<String>,

        /// Working directory for the command(s)
        #[arg(short, long)]
        cwd: Option<PathBuf>,

        /// Environment variables (KEY=VALUE format)
        #[arg(short, long)]
        env: Vec<String>,
    },

    /// List all background processes
    List {
        /// Filter by status (running, exited, killed)
        #[arg(short, long)]
        status: Option<String>,
    },

    /// Show logs for a process
    Log {
        /// Task ID to show logs for
        task_id: String,

        /// Follow log output (like tail -f)
        #[arg(short, long)]
        follow: bool,
    },

    /// Stop a background process
    Stop {
        /// Task ID to stop
        task_id: String,

        /// Force kill the process (SIGKILL instead of SIGTERM)
        #[arg(short, long)]
        force: bool,
    },

    /// Check status of a background process
    Status {
        /// Task ID to check
        task_id: String,
    },

    /// Clean up old finished tasks
    Cleanup {
        /// Delete tasks older than this many days (default: 30)
        #[arg(short, long, default_value = "30")]
        days: u64,

        /// Filter by status (exited, killed, all). Default: exited,killed
        #[arg(short, long)]
        status: Option<String>,

        /// Show what would be deleted without actually deleting
        #[arg(short = 'n', long)]
        dry_run: bool,

        /// Delete all finished tasks regardless of age
        #[arg(short, long)]
        all: bool,
    },

    /// Run MCP server for ghost operations
    Mcp,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(cmd) => {
            // Initialize database connection once for all commands (except TUI)
            match storage::init_database() {
                Ok(conn) => match cmd {
                    Commands::Run { commands, cwd, env } => run_commands(&conn, commands, cwd, env),
                    Commands::List { status } => commands::list(&conn, status, true).map(|_| ()),
                    Commands::Log { task_id, follow } => {
                        commands::log(&conn, &task_id, follow, true)
                            .await
                            .map(|_| ())
                    }
                    Commands::Stop { task_id, force } => {
                        commands::stop(&conn, &task_id, force, true)
                    }
                    Commands::Status { task_id } => {
                        commands::status(&conn, &task_id, true).map(|_| ())
                    }
                    Commands::Cleanup {
                        days,
                        status,
                        dry_run,
                        all,
                    } => commands::cleanup(&conn, days, status, dry_run, all),
                    Commands::Mcp => {
                        // Initialize file logger for MCP server
                        let log_dir = config::get_log_dir();
                        let _guard = logging::init_file_logger(&log_dir);

                        ghost::mcp::run_stdio_server(conn).await.map_err(|e| {
                            ghost::app::error::GhostError::Config {
                                message: e.to_string(),
                            }
                        })
                    }
                },
                Err(e) => Err(e),
            }
        }
        None => commands::tui().await,
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

/// Run one or more commands based on the input format
///
/// Determines whether to use single-command mode (backward compatible)
/// or multi-command mode based on whether the first argument contains spaces.
fn run_commands(
    conn: &Connection,
    args: Vec<String>,
    cwd: Option<PathBuf>,
    env: Vec<String>,
) -> Result<()> {
    if args.is_empty() {
        return Err(ghost::app::error::GhostError::InvalidArgument {
            message: "No command specified".to_string(),
        });
    }

    // Determine if this is multi-command format:
    // - If first arg contains space -> multi-command format (each arg is a full command)
    // - Otherwise -> single-command format (all args form one command)
    let is_multi_command = args.first().map(|s| s.contains(' ')).unwrap_or(false);

    if is_multi_command {
        // Multi-command mode: each argument is a complete command string
        // Note: Error messages are printed by spawn_multi, so we don't need to handle failures here
        let _ = commands::spawn_multi(conn, args, cwd, env, true);
        Ok(())
    } else {
        // Single-command mode: all arguments form one command (backward compatible)
        commands::spawn(conn, args, cwd, env, true).map(|_| ())
    }
}
