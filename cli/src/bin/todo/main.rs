mod args;
mod command_error;
mod command_impl;
mod command_impl_cloud;
mod utils;

use args::{Command, TodoArgs};
use clap::Parser;
use console::Style;
use std::io::ErrorKind;
use yesser_todo_api::Client;
use yesser_todo_db::{SaveData, Task, get_index};

use crate::utils::process_cloud_config;

#[tokio::main]
async fn main() {
    let args = TodoArgs::parse();
    let mut data = SaveData::new();
    // let done_style = Style::new().strikethrough().green();

    match data.load_tasks() {
        Ok(_) => {}
        Err(err) => {
            println!("Error while getting saved data: {err}");
            return;
        }
    }

    let mut client: Option<Client> = None;

    match process_cloud_config() {
        Some((hostname, port)) => {
            client = Some(Client::new(hostname, Some(port)));
        }
        None => {}
    }

    match args.command.execute(data.get_tasks(), &mut client).await {
        Ok(()) => match args.command {
            Command::List => {}
            _ => match Command::List.execute(data.get_tasks(), &mut client).await {
                Ok(()) => {}
                Err(err) => err.handle(),
            },
        },
        Err(err) => err.handle(),
    }
}

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
async fn old_main() {
    let args = TodoArgs::parse();
    let mut data = SaveData::new();
    let done_style = Style::new().strikethrough().green();

    let _ = data.load_tasks();

    match &args.command {
        Command::Add(command) => {
            if command.tasks.len() <= 0 {
                println!("No tasks specified!")
            } else {
                match process_cloud_config() {
                    None => {
                        for task in &command.tasks {
                            let option = get_index(data.get_tasks(), task);
                            match option {
                                Some(_) => {
                                    // Task already exists
                                    println!("Task {task} already exists!");
                                }
                                None => {
                                    let task_obj: Task = Task {
                                        name: task.clone(),
                                        done: false,
                                    };
                                    data.add_task(task_obj);
                                }
                            }
                        }
                    }
                    Some((host, port)) => {
                        let client = Client::new(host, Some(port));
                        for task in &command.tasks {
                            let result = client.get_index(task).await;
                            let mut exists: bool = false;
                            match result {
                                Ok((status_code, _)) => {
                                    if status_code.is_success() {
                                        exists = true
                                    }
                                }
                                Err(_) => {}
                            }
                            match exists {
                                true => {
                                    // Task already exists
                                    println!("Task {task} already exists!");
                                }
                                false => {
                                    let result = client.add(task).await;
                                    match result {
                                        Ok((status_code, _)) => {
                                            if status_code.is_success() {
                                                println!("Task {task} added successfully!");
                                            } else {
                                                println!(
                                                    "HTTP Error while adding task {task}: {}!",
                                                    status_code.as_u16()
                                                );
                                            }
                                        }
                                        Err(err) => {
                                            println!("Adding task {task} failed! {err}");
                                        }
                                    }
                                }
                            }
                        }
                    }
                };
            }
        }
        Command::Remove(command) => {
            if command.tasks.len() <= 0 {
                println!("No tasks specified!")
            } else {
                match process_cloud_config() {
                    None => {
                        for task in &command.tasks {
                            let option = get_index(data.get_tasks(), task);
                            match option {
                                Some(index) => {
                                    data.remove_task(index);
                                }
                                None => println!("Unable to find task {task}!"),
                            }
                        }
                    }
                    Some((host, port)) => {
                        let client = Client::new(host, Some(port));
                        for task in &command.tasks {
                            let result = client.remove(task).await;
                            match result {
                                Ok(status_code) => {
                                    if status_code.is_success() {
                                        println!("Task {task} removed!");
                                    } else if status_code.as_u16() == 404 {
                                        println!("Task {task} not found!");
                                    } else {
                                        println!(
                                            "Error while removing task {task}: {status_code}!"
                                        );
                                    }
                                }
                                Err(err) => println!(
                                    "Removing task {task} failed (task may still exist): {err}"
                                ),
                            }
                        }
                    }
                }
            }
        }
        Command::Done(command) => {
            if command.tasks.len() <= 0 {
                println!("No tasks specified!")
            } else {
                match process_cloud_config() {
                    None => {
                        for task in &command.tasks {
                            let option = get_index(data.get_tasks(), task);
                            match option {
                                Some(index) => {
                                    data.mark_task_done(index);
                                }
                                None => println!("Unable to find task {task}!"),
                            }
                        }
                    }
                    Some((host, port)) => {
                        let client = Client::new(host, Some(port));
                        for task in &command.tasks {
                            let result = client.done(task).await;
                            match result {
                                Ok((status_code, _)) => {
                                    if status_code.is_success() {
                                        println!("Task {task} done!");
                                    } else if status_code.as_u16() == 404 {
                                        println!("Task {task} not found!");
                                    } else {
                                        println!(
                                            "HTTP Error while marking task {task} as done: {status_code}!"
                                        );
                                    }
                                }
                                Err(err) => {
                                    println!("Marking task {task} as done failed: {err}")
                                }
                            }
                        }
                    }
                }
            }
        }
        Command::Undone(command) => {
            if command.tasks.len() <= 0 {
                println!("No tasks specified!")
            } else {
                match process_cloud_config() {
                    None => {
                        for task in &command.tasks {
                            let option = get_index(data.get_tasks(), task);
                            match option {
                                Some(index) => {
                                    data.mark_task_undone(index);
                                }
                                None => println!("Unable to find task {task}!"),
                            }
                        }
                    }
                    Some((host, port)) => {
                        let client = Client::new(host, Some(port));
                        for task in &command.tasks {
                            let result = client.undone(task).await;
                            match result {
                                Ok((status_code, _)) => {
                                    if status_code.is_success() {
                                        println!("Task {task} undone!");
                                    } else if status_code.as_u16() == 404 {
                                        println!("Task {task} not found!");
                                    } else {
                                        println!(
                                            "HTTP Error while marking task {task} as undone: {status_code}!"
                                        );
                                    }
                                }
                                Err(err) => {
                                    println!("Marking task {task} as undone failed: {err}")
                                }
                            }
                        }
                    }
                }
            }
        }
        Command::Clear(command) => match process_cloud_config() {
            None => {
                if command.done {
                    data.clear_done_tasks();
                } else {
                    data.clear_tasks()
                }
            }
            Some((host, port)) => {
                let client = Client::new(host, Some(port));
                let result: Result<_, _>;
                if command.done {
                    result = client.clear_done().await;
                } else {
                    result = client.clear().await;
                }
                match result {
                    Ok(status_code) => {
                        if status_code.is_success() {
                            println!("Tasks cleared!");
                        } else {
                            println!("HTTP error while clearing: {}", status_code.as_u16());
                        }
                    }
                    Err(err) => println!("Clearing tasks failed: {err}"),
                }
            }
        },
        Command::ClearDone => {
            println!("clear-done is deprecated. Use clear -d instead.");

            match process_cloud_config() {
                None => data.clear_done_tasks(),
                Some((host, port)) => {
                    let client = Client::new(host, Some(port));
                    let result = client.clear_done().await;
                    match result {
                        Ok(status_code) => {
                            if status_code.is_success() {
                                println!("Tasks cleared!");
                            } else {
                                println!("HTTP error while clearing: {}", status_code.as_u16());
                            }
                        }
                        Err(err) => println!("Clearing tasks failed: {err}"),
                    }
                }
            }
        }
        Command::List => {} // List just shows the tasks, that is below
        Command::Connect(command) => {
            let result = match &command.port {
                None => {
                    let client = Client::new("".to_string(), None);
                    SaveData::save_cloud_config(&command.host, &client.port)
                }
                Some(port) => SaveData::save_cloud_config(&command.host, port),
            };
            match result {
                Ok(_) => println!("Successfully linked server."),
                Err(_) => {
                    println!("Unable to save server configuration.")
                }
            }
        }
        Command::Disconnect => {
            let result = SaveData::remove_cloud_config();
            match result {
                Ok(_) => {
                    println!("Successfully unlinked server.")
                }
                Err(err) => match err.kind() {
                    ErrorKind::NotFound => println!("You're already unlinked!"),
                    _ => println!("Something went wrong"),
                },
            }
        }
    }

    match process_cloud_config() {
        None => {
            data.save_tasks().unwrap();

            println!("\nCurrent tasks:");
            for task in data.get_tasks() {
                if task.done {
                    println!("{}", done_style.apply_to(&task.name))
                } else {
                    println!("{}", task.name)
                }
            }
        }
        Some((host, port)) => {
            let client = Client::new(host, Some(port));
            let result = client.get().await;
            match result {
                Ok((status_code, tasks)) => {
                    if status_code.is_success() {
                        println!("\nCurrent tasks:");
                        for task in tasks {
                            if task.done {
                                println!("{}", done_style.apply_to(&task.name))
                            } else {
                                println!("{}", task.name)
                            }
                        }
                    } else {
                        println!("HTTP error while getting tasks: {}", status_code.as_u16());
                    }
                }
                Err(err) => println!("Error while getting tasks: {err}"),
            }
        }
    }
}
