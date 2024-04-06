use clap::Parser;

#[derive(Parser)]
#[command(version, about = "Command line Pomodoro Timer", long_about = None)]
struct Args {
    #[arg(short, long, help="add <TASK> to the tracker", value_name = "TASK" )]
    add: String,

    #[arg(short, long, help="delete <TASK> from the tracker", value_name = "TASK" )]
    delete: String,

    #[arg(long, help="start the tracker for <TASK>", value_name = "TASK" )]
    start: String,

    #[arg(long, help="stops currently running tracker")]
    stop: bool,

    #[arg(short, long, help="list accumulated time for <TASK>", default_value = "all", value_name = "TASK")]
    log: String,
}

fn main() {
    let cli = Args::parse();
    
    println!("hello");
}