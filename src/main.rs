use clap::{Parser, Subcommand};
use std::path::PathBuf;

use ghost::app::commands;

#[derive(Parser, Debug)]
#[command(name = "ghost")]
#[command(about = "A simple background process manager")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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

    /// Start TUI mode for interactive task management
    Tui,
}

#[tokio::main]
async fn main() {
    if cfg!(windows) {
        eprintln!("ghost does not support Windows yet.");
        std::process::exit(1);
    }

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Run { command, cwd, env } => commands::spawn(command, cwd, env),
        Commands::List { status } => commands::list(status),
        Commands::Log { task_id, follow } => commands::log(&task_id, follow).await,
        Commands::Stop { task_id, force } => commands::stop(&task_id, force),
        Commands::Status { task_id } => commands::status(&task_id),
        Commands::Cleanup {
            days,
            status,
            dry_run,
            all,
        } => commands::cleanup(days, status, dry_run, all),
        Commands::Tui => commands::tui().await,
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
