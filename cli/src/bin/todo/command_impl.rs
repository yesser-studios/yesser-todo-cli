use std::collections::HashSet;

use crate::{args::ClearCommand, utils::DONE_STYLE};
use yesser_todo_db::{Task, get_index};

use crate::{args::TasksCommand, command_error::CommandError};

/// Adds each task named in `command.tasks` to `data` after validating input.
///
/// Validations performed:
/// - Returns `CommandError::NoTasksSpecified` if no tasks are provided.
/// - Returns `CommandError::DuplicateInput { name }` if the same name appears more than once in the input.
/// - Returns `CommandError::TaskExists { name }` if any input name already exists in `data`.
///
/// # Examples
///
/// ```
/// // Assume `TasksCommand`, `Task`, and `CommandError` are in scope.
/// let mut data = Vec::new();
/// let cmd = TasksCommand { tasks: vec!["task1".into(), "task2".into()] };
/// handle_add(&cmd, &mut data).unwrap();
/// assert_eq!(data.len(), 2);
/// assert_eq!(data[0].name, "task1");
/// assert_eq!(data[1].name, "task2");
/// ```
pub(crate) fn handle_add(command: &TasksCommand, data: &mut Vec<Task>) -> Result<(), CommandError> {
    if command.tasks.is_empty() {
        return Err(CommandError::NoTasksSpecified);
    }

    let mut seen = HashSet::new();
    for task in &command.tasks {
        if !seen.insert(task.as_str()) {
            return Err(CommandError::DuplicateInput { name: task.clone() });
        } else if data.iter().any(|x| x.name == task.as_str()) {
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

/// Remove the specified tasks from the in-memory task list.
///
/// This validates the input list and removes each named task from `data` in order.
///
/// # Errors
///
/// Returns `CommandError::NoTasksSpecified` if `command.tasks` is empty.
/// Returns `CommandError::DuplicateInput { name }` if `command.tasks` contains a duplicate name.
/// Returns `CommandError::TaskNotFound { name }` if any specified task does not exist in `data`.
///
/// # Examples
///
/// ```
/// use yesser_todo_db::Task;
/// use crate::TasksCommand;
///
/// let mut data = vec![Task { name: "one".into(), done: false }, Task { name: "two".into(), done: false }];
/// let cmd = TasksCommand { tasks: vec!["one".into()] };
/// let res = handle_remove(&cmd, &mut data);
/// assert!(res.is_ok());
/// assert_eq!(data.len(), 1);
/// assert_eq!(data[0].name, "two");
/// ```
pub(crate) fn handle_remove(command: &TasksCommand, data: &mut Vec<Task>) -> Result<(), CommandError> {
    if command.tasks.is_empty() {
        return Err(CommandError::NoTasksSpecified);
    }

    let mut seen = HashSet::new();
    for task in &command.tasks {
        if !seen.insert(task.as_str()) {
            return Err(CommandError::DuplicateInput { name: task.clone() });
        } else if get_index(data, task).is_none() {
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

/// Prints the current list of tasks to stdout, applying the configured visual style to completed tasks.
///
/// Each task is printed on its own line and a header "Current tasks:" is printed before the list.
///
/// # Examples
///
/// ```
/// let tasks = vec![
///     Task { name: String::from("buy milk"), done: false },
///     Task { name: String::from("wash car"), done: true },
/// ];
/// handle_list(&tasks).unwrap();
/// ```
///
/// Returns `Ok(())` on success.
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

/// Marks the specified tasks as done or undone.
///
/// Validates that `command.tasks` is non-empty, contains no duplicate names, and that each named task exists in `data`.
///
/// # Parameters
///
/// - `command`: The task names to update (from `TasksCommand::tasks`).
/// - `data`: Mutable task list to update.
/// - `done`: If `true`, mark matching tasks as done; if `false`, mark them as not done.
///
/// # Returns
///
/// `Ok(())` on success; `Err(CommandError::NoTasksSpecified)` if no tasks were provided, `Err(CommandError::DuplicateInput { name })` if the input contains duplicate names, or `Err(CommandError::TaskNotFound { name })` if a named task does not exist.
///
/// # Examples
///
/// ```
/// use yesser_todo_db::Task;
/// use crate::command_impl::handle_done_undone;
/// use crate::commands::TasksCommand;
///
/// let mut tasks = vec![Task { name: "a".into(), done: false }];
/// let cmd = TasksCommand { tasks: vec!["a".into()] };
/// let res = handle_done_undone(&cmd, &mut tasks, true);
/// assert!(res.is_ok());
/// assert!(tasks[0].done);
/// ```
pub(crate) fn handle_done_undone(command: &TasksCommand, data: &mut Vec<Task>, done: bool) -> Result<(), CommandError> {
    if command.tasks.is_empty() {
        return Err(CommandError::NoTasksSpecified);
    }

    let mut seen = HashSet::new();
    for task in &command.tasks {
        if !seen.insert(task.as_str()) {
            return Err(CommandError::DuplicateInput { name: task.clone() });
        } else if get_index(data, task).is_none() {
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

/// Removes tasks from `data` according to `command`.
///
/// If `command.done` is `true`, retains only tasks that are not done (removes completed tasks).
/// If `command.done` is `false`, clears all tasks from `data`.
///
/// # Examples
///
/// ```
/// let mut data = vec![
///     Task { name: String::from("one"), done: true },
///     Task { name: String::from("two"), done: false },
/// ];
/// handle_clear(&ClearCommand { done: true }, &mut data).unwrap();
/// assert_eq!(data.len(), 1);
/// assert_eq!(data[0].name, "two");
/// ```
///
/// # Returns
///
/// `Ok(())` on success.
pub(crate) fn handle_clear(command: &ClearCommand, data: &mut Vec<Task>) -> Result<(), CommandError> {
    if command.done {
        data.retain(|t| !t.done);
    } else {
        data.clear();
    }
    Ok(())
}

/// Clears tasks that are marked as done.
///
/// Deprecated: prefer using the `clear -d` command instead.
///
/// # Examples
///
/// ```
/// let mut data = vec![
///     Task { name: "done-task".into(), done: true },
///     Task { name: "keep-task".into(), done: false },
/// ];
/// let _ = handle_clear_done(&mut data);
/// assert_eq!(data.len(), 1);
/// assert_eq!(data[0].name, "keep-task");
/// ```
///
/// # Returns
///
/// `Ok(())` on success, or a `CommandError` if an error occurs.
#[deprecated]
pub(crate) fn handle_clear_done(data: &mut Vec<Task>) -> Result<(), CommandError> {
    println!("clear-done is deprecated. Use clear -d instead.");
    handle_clear(&ClearCommand { done: true }, data)
}
