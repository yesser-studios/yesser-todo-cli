mod args;
mod db;

use std::ops::Deref;
use clap::Parser;
use args::{TodoArgs,Command};
use db::{SaveData,Task,get_index};
use console::Style;

fn main() {
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
                for task in &command.tasks {
                    let option = get_index(data.get_tasks(), task);
                    match option {
                        Some(_) => {
                            // Task already exists
                            println!("Task already exists!");
                            success = false;
                        }
                        None => {
                            let task_obj: Task = Task{name: task.deref().parse().unwrap(), done: false};
                            data.add_task(task_obj);
                        }
                    }
                }
                if success {
                    println!("Successfully added tasks.")
                }
            }
        }
        Command::Remove(command) => {
            if command.tasks.len() <= 0 {
                println!("No tasks specified!")
            } else {
                for task in &command.tasks {
                    let option = get_index(data.get_tasks(), task);
                    match option {
                        Some(index) => {
                            data.remove_task(index);
                        }
                        None => println!("Unable to find specified task!"),
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
            data.clear_tasks();
        }
        Command::List => {} // List just shows the tasks, that is below:
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
