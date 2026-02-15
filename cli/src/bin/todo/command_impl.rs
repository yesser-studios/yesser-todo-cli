use yesser_todo_db::Task;

use crate::{args::TasksCommand, command_error::CommandError};

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
