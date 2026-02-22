use crate::utils::DONE_STYLE;
use yesser_todo_db::{Task, get_index};

use crate::{args::TasksCommand, command_error::CommandError};

pub(crate) fn handle_add(command: &TasksCommand, data: &mut Vec<Task>) -> Result<(), CommandError> {
    if command.tasks.len() <= 0 {
        return Err(CommandError::NoTasksSpecified);
    }

    for task in &command.tasks {
        if data.iter().find(|x| x.name == task.as_str()).is_some() {
            return Err(CommandError::TaskExists { name: task.clone() });
        }
        let task_obj: Task = Task {
            name: task.clone(),
            done: false,
        };
        data.push(task_obj);
    }
    Ok(())
}

pub(crate) fn handle_remove(
    command: &TasksCommand,
    data: &mut Vec<Task>,
) -> Result<(), CommandError> {
    if command.tasks.len() <= 0 {
        return Err(CommandError::NoTasksSpecified);
    }

    for task in &command.tasks {
        match get_index(data, task) {
            Some(index) => {
                data.remove(index);
            }
            None => return Err(CommandError::TaskNotFound { name: task.clone() }),
        }
    }
    Ok(())
}

pub(crate) fn handle_list(data: &Vec<Task>) -> Result<(), CommandError> {
    println!("\nCurrent tasks:");
    for task in data {
        if task.done {
            println!("{}", DONE_STYLE.apply_to(task.name.as_str()));
        } else {
            println!("{}", task.name);
        }
    }

    Ok(())
}

pub(crate) fn handle_done_undone(
    command: &TasksCommand,
    data: &mut Vec<Task>,
    done: bool,
) -> Result<(), CommandError> {
    if command.tasks.len() <= 0 {
        return Err(CommandError::NoTasksSpecified);
    }

    for task in &command.tasks {
        match get_index(data, task) {
            Some(index) => data[index].done = done,
            None => return Err(CommandError::TaskNotFound { name: task.clone() }),
        }
    }

    Ok(())
}
