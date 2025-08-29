use clap::{Parser, Subcommand};
use std::path::PathBuf;

use ghost::app::commands;

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

        /// Show all tasks (default: last 24 hours)
        #[arg(short, long)]
        all: bool,
    },

    /// Show logs for a process
    Log {
        /// Task ID to show logs for
        task_id: String,

        /// Follow log output (like tail -f)
        #[arg(short, long)]
        follow: bool,

        /// Show all log content (default: head 100 + tail 100)
        #[arg(short, long)]
        all: bool,

        /// Number of lines from the beginning (default: 100, ignored if --all)
        #[arg(long, default_value = "100")]
        head: usize,

        /// Number of lines from the end (default: 100, ignored if --all)
        #[arg(long, default_value = "100")]
        tail: usize,
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

    /// Restart a background process
    Restart {
        /// Task ID to restart
        task_id: String,

        /// Force kill the process (SIGKILL instead of SIGTERM)
        #[arg(short, long)]
        force: bool,
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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Run { command, cwd, env }) => commands::spawn(command, cwd, env),
        Some(Commands::List { status, all }) => commands::list(status, all),
        Some(Commands::Log {
            task_id,
            follow,
            all,
            head,
            tail,
        }) => commands::log(&task_id, follow, all, head, tail).await,
        Some(Commands::Stop { task_id, force }) => commands::stop(&task_id, force, true),
        Some(Commands::Status { task_id }) => commands::status(&task_id),
        Some(Commands::Restart { task_id, force }) => commands::restart(&task_id, force),
        Some(Commands::Cleanup {
            days,
            status,
            dry_run,
            all,
        }) => commands::cleanup(days, status, dry_run, all),
        None => commands::tui().await, // No subcommand = start TUI
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
