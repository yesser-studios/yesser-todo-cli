mod args;
mod db;

use std::ops::Deref;
use clap::{Parser};
use args::{TodoArgs,Command};
use db::{SaveData,Task};
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
                for task in &command.tasks {
                    let task_obj: Task = Task{name: task.deref().parse().unwrap(), done: false};
                    data.add_task(task_obj);
                }
                println!("Successfully added tasks.")
            }
        }
        Command::Remove(command) => {
            if command.tasks.len() <= 0 {
                println!("No tasks specified!")
            } else {
                for task in &command.tasks {
                    let index = data.get_tasks()
                        .iter().position(|r| &r.name == task).unwrap();

                    data.remove_task(index);
                }
            }
        }
        Command::Done(command) => {
            if command.tasks.len() <= 0 {
                println!("No tasks specified!")
            } else {
                for task in &command.tasks {
                    let index = data.get_tasks()
                        .iter().position(|r| &r.name == task).unwrap();

                    data.mark_task_done(index);
                }
            }
        }
        Command::Undone(command) => {
            if command.tasks.len() <= 0 {
                println!("No tasks specified!")
            } else {
                for task in &command.tasks {
                    let index = data.get_tasks()
                        .iter().position(|r| &r.name == task).unwrap();

                    data.mark_task_undone(index);
                }
            }
        }
        Command::Clear => {
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
