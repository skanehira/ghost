use clap::{Parser, Subcommand};
use std::path::PathBuf;

use ghost::app::{commands, storage};

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
    /// Run a command in the background
    Run {
        /// The command to run
        command: Vec<String>,

        /// Working directory for the command
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
                    Commands::Run { command, cwd, env } => {
                        commands::spawn(&conn, command, cwd, env, true).map(|_| ())
                    }
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
                    Commands::Mcp => ghost::mcp::run_stdio_server(conn).await.map_err(|e| {
                        ghost::app::error::GhostError::Config {
                            message: e.to_string(),
                        }
                    }),
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
