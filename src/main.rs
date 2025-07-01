use clap::Parser;
use ghost::core::command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    command: Vec<String>,
    #[arg(short, long, help = "Process ID to kill")]
    kill: Option<u32>,
}

fn main() {
    if cfg!(windows) {
        eprintln!("ghost does not support Windows yet.");
        return;
    }

    let args = Args::parse();

    if let Some(pid) = args.kill {
        match command::kill_process(pid, true) {
            Ok(_) => {
                println!("Process {pid} killed successfully.");
            }
            Err(e) => {
                eprintln!("Error killing process {pid}: {e}");
            }
        }
        return;
    };

    match command::spawn_background_process(args.command, None) {
        Ok(result) => {
            println!("Process started successfully: {result:?}");
        }
        Err(e) => {
            eprintln!("Error starting process: {e}");
        }
    }
}
