mod args;
mod command_impl;
mod command_impl_cloud;
mod db_error_wrap;
mod utils;

use args::{Command, TodoArgs};
use clap::Parser;
use yesser_todo_api::Client;
use yesser_todo_db::{JsonSaveData, SaveData};

use crate::utils::get_client;

/// Application entry point for the Todo CLI.
///
/// Parses command-line arguments, loads local task data, and executes the requested command.
/// Commands operate either on local storage or a remote server depending on the saved cloud
/// configuration. After executing a command the current task list is displayed with completed
/// tasks rendered using the configured "done" style.
///
/// # Examples
///
/// ```text
/// $ todo add "Buy milk"
/// ```
fn main() {
    let args = TodoArgs::parse();
    let mut data: Box<dyn SaveData> = match JsonSaveData::new() {
        Ok(data) => Box::new(data),
        Err(err) => {
            eprintln!("Error while getting saved data: {err}");
            return;
        }
    };

    match data.load_tasks() {
        Ok(_) => {}
        Err(err) => {
            println!("Error while getting saved data: {err}");
            return;
        }
    }

    let mut client: Option<Client> = get_client(Some(&args), &*data);

    match args.command.execute(&mut *data, &mut client) {
        Ok(()) => match args.command {
            Command::List => {}
            Command::Cloud(_) | Command::Connect(_) | Command::Disconnect => {}
            _ => {
                if client.is_none() {
                    match data.save_tasks() {
                        Ok(()) => {}
                        Err(err) => {
                            println!("Failed to save tasks: {}", err);
                            return;
                        }
                    }
                }

                match Command::List.execute(&mut *data, &mut client) {
                    Ok(()) => {}
                    Err(err) => err.handle(),
                }
            }
        },
        Err(err) => err.handle(),
    }
}
