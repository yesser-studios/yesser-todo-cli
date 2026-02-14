use yesser_todo_db::{SaveData, Task, get_index};

use crate::{args::TasksCommand, command_error::CommandError};

/// Retrieve the saved cloud server host and port if available.
///
/// # Returns
///
/// `Some((host, port))` when a cloud configuration is present and readable, `None` if no configuration exists or it cannot be read.
///
/// # Examples
///
/// ```
/// // Suppose SaveData::get_cloud_config() returns Ok(Some(("example.com".into(), "6982".into())))
/// if let Some((host, port)) = process_cloud_config() {
///     assert_eq!(host, "example.com");
///     assert_eq!(port, "6982");
/// } else {
///     panic!("expected cloud config");
/// }
/// ```
pub(crate) fn process_cloud_config() -> Option<(String, String)> {
    SaveData::get_cloud_config().unwrap_or_else(|_| None)
}

pub(crate) fn handle_add(command: &TasksCommand, data: &mut Vec<Task>) -> Result<(), CommandError> {
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
