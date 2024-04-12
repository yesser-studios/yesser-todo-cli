mod args;

use clap::{Parser};
use args::{TodoArgs,Command};

fn main() {
    let args = TodoArgs::parse();

    match &args.command {
        Command::Add(command) => {
            println!("Tasks: {:?}", command.tasks.as_slice())
        }
        _ => {}
    }
}
