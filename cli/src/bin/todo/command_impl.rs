use crate::{args::ClearCommand, utils::DONE_STYLE};
use yesser_todo_db::{Task, get_index};

use crate::{args::TasksCommand, command_error::CommandError};

pub(crate) fn handle_add(command: &TasksCommand, data: &mut Vec<Task>) -> Result<(), CommandError> {
    if command.tasks.is_empty() {
        return Err(CommandError::NoTasksSpecified);
    }

    for task in &command.tasks {
        if data.iter().any(|x| x.name == task.as_str()) {
            return Err(CommandError::TaskExists { name: task.clone() });
        }
    }

    for task in &command.tasks {
        let task_obj: Task = Task {
            name: task.clone(),
            done: false,
        };
        data.push(task_obj);
    }
    Ok(())
}

pub(crate) fn handle_remove(command: &TasksCommand, data: &mut Vec<Task>) -> Result<(), CommandError> {
    if command.tasks.len() <= 0 {
        return Err(CommandError::NoTasksSpecified);
    }

    for task in &command.tasks {
        if get_index(data, task).is_none() {
            return Err(CommandError::TaskNotFound { name: task.clone() });
        }
    }

    for task in &command.tasks {
        if let Some(index) = get_index(data, task) {
            data.remove(index);
        }
    }
    Ok(())
}

pub(crate) fn handle_list(data: &[Task]) -> Result<(), CommandError> {
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

pub(crate) fn handle_done_undone(command: &TasksCommand, data: &mut Vec<Task>, done: bool) -> Result<(), CommandError> {
    if command.tasks.len() <= 0 {
        return Err(CommandError::NoTasksSpecified);
    }

    for task in &command.tasks {
        if get_index(data, task).is_none() {
            return Err(CommandError::TaskNotFound { name: task.clone() });
        }
    }

    for task in &command.tasks {
        if let Some(index) = get_index(data, task) {
            data[index].done = done;
        }
    }

    Ok(())
}

pub(crate) fn handle_clear(command: &ClearCommand, data: &mut Vec<Task>) -> Result<(), CommandError> {
    if command.done {
        data.retain(|t| !t.done);
    } else {
        data.clear();
    }
    Ok(())
}

#[deprecated]
pub(crate) fn handle_clear_done(data: &mut Vec<Task>) -> Result<(), CommandError> {
    println!("clear-done is deprecated. Use clear -d instead.");
    handle_clear(&ClearCommand { done: true }, data)
}
