mod args;

use args::{Command, TodoArgs};
use clap::Parser;
use console::Style;
use std::io::ErrorKind;
use std::ops::Deref;
use yesser_todo_api::Client;
use yesser_todo_db::{get_index, SaveData, Task};

fn process_cloud_config() -> Option<(String, String)> {
    match SaveData::get_cloud_config() {
        Ok(option) => match option {
            None => None,
            Some((host, port)) => Some((host, port)),
        }
        Err(_) => None,
    }
}

#[tokio::main]
async fn main() {
    let args = TodoArgs::parse();
    let mut data = SaveData::new();
    let done_style = Style::new().strikethrough().green();

    let _ = data.load_tasks();

    match &args.command {
        Command::Add(command) => {
            if command.tasks.len() <= 0 {
                println!("No tasks specified!")
            } else {
                let mut success = true;
                match process_cloud_config() {
                    None => {
                        for task in &command.tasks {
                            let option = get_index(data.get_tasks(), task);
                            match option {
                                Some(_) => {
                                    // Task already exists
                                    println!("Task {task} already exists!");
                                    success = false;
                                }
                                None => {
                                    let task_obj: Task = Task { name: task.deref().parse().unwrap(), done: false };
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
                                    success = false;
                                }
                                false => {
                                    let result = client.add(task).await;
                                    match result {
                                        Ok(_) => {}
                                        Err(_) => {
                                            println!("Adding task {task} failed!");
                                            success = false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                };
                if success {
                    println!("Successfully added tasks.")
                }
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
                                        println!("Task removed!");
                                    }
                                    if status_code.as_u16() == 404 {
                                        println!("Task {task} not found!");
                                    }
                                }
                                Err(_) => println!("Removing task {task} failed (task may still exist)."),
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
                for task in &command.tasks {
                    let option = get_index(data.get_tasks(), task);
                    match option {
                        Some(index) => {
                            data.mark_task_done(index);
                        }
                        None => println!("Unable to find specified task!"),
                    }
                }
            }
        }
        Command::Undone(command) => {
            if command.tasks.len() <= 0 {
                println!("No tasks specified!")
            } else {
                for task in &command.tasks {
                    let option = get_index(data.get_tasks(), task);
                    match option {
                        Some(index) => {
                            data.mark_task_undone(index);
                        }
                        None => println!("Unable to find specified task!"),
                    }
                }
            }
        }
        Command::Clear => {
            data.clear_tasks();
        }
        Command::ClearDone => {
            data.clear_done_tasks();
        }
        Command::List => {} // List just shows the tasks, that is below:
        Command::Connect(command) => {
            let result = match &command.port {
                None => SaveData::save_cloud_config(&command.host, &"6982".to_string()),
                Some(port) => SaveData::save_cloud_config(&command.host, port),
            };
            match result {
                Ok(_) => println!("Successfully linked server."),
                Err(_) => {println!("Unable to save server configuration.")}
            }
        }
        Command::Disconnect => {
            let result = SaveData::remove_cloud_config();
            match result {
                Ok(_) => {println!("Successfully unlinked server.")}
                Err(err) => {
                    match err.kind() {
                        ErrorKind::NotFound => println!("You're already unlinked!"),
                        _ => println!("Something went wrong")
                    }
                }
            }
        }
    }

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
