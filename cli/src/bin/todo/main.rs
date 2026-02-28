mod args;
mod command_error;
mod command_impl;
mod command_impl_cloud;
mod utils;

use args::{Command, TodoArgs};
use clap::Parser;
use yesser_todo_api::Client;
use yesser_todo_db::SaveData;

use crate::utils::process_cloud_config;

/// Application entry point for the Todo CLI.
///
/// Parses command-line arguments, loads local task data, and executes the requested command.
/// Commands operate either on local storage or a remote server depending on the saved cloud
/// configuration. After executing a command the current task list is displayed with completed
/// tasks rendered using the configured "done" style.
///
/// # Examples
///
/// ```no_run
/// // Run the CLI binary (example):
/// // $ todo add "Buy milk"
/// ```
#[tokio::main]
async fn main() {
    let args = TodoArgs::parse();
    let mut data = SaveData::new();

    match data.load_tasks() {
        Ok(_) => {}
        Err(err) => {
            println!("Error while getting saved data: {err}");
            return;
        }
    }

    let mut client: Option<Client> = None;

    if let Some((hostname, port)) = process_cloud_config() {
        client = Some(Client::new(hostname, Some(port)));
    }

    match args.command.execute(data.get_tasks(), &mut client).await {
        Ok(()) => match args.command {
            Command::List => {}
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

                match Command::List.execute(data.get_tasks(), &mut client).await {
                    Ok(()) => {}
                    Err(err) => err.handle(),
                }
            }
        },
        Err(err) => err.handle(),
    }
}
