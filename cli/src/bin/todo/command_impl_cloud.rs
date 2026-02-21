use yesser_todo_api::Client;

use crate::{args::TasksCommand, command_error::CommandError};

pub(crate) async fn check_exists_cloud(task: &str, client: &Client) -> Result<bool, CommandError> {
    let result = client.get_index(task).await;
    match result {
        Ok((status_code, _)) => {
            if status_code.is_success() {
                return Ok(true);
            } else if status_code.as_u16() == 404 {
                return Ok(false);
            } else {
                return Err(CommandError::HTTPError {
                    name: task.to_string(),
                    status_code: status_code.as_u16(),
                });
            }
        }
        Err(_) => Err(CommandError::ConnectionError {
            name: task.to_string(),
        }),
    }
}

pub(crate) async fn handle_add_cloud(
    command: &TasksCommand,
    client: &mut Client,
) -> Result<(), CommandError> {
    let results = command.tasks.iter().map(|task| (client.add(task), task));
    for (result, task) in results {
        match check_exists_cloud(task, client).await {
            Ok(true) => return Err(CommandError::TaskExists { name: task.clone() }),
            Ok(false) => {}
            Err(err) => return Err(err),
        }
        match result.await {
            Err(_) => return Err(CommandError::ConnectionError { name: task.clone() }),
            Ok((status_code, _)) => {
                if !status_code.is_success() {
                    return Err(CommandError::HTTPError {
                        name: task.clone(),
                        status_code: status_code.as_u16(),
                    });
                }
            }
        };
    }
    Ok(())
}

pub(crate) async fn handle_remove_cloud(
    command: &TasksCommand,
    client: &mut Client,
) -> Result<(), CommandError> {
    let results = command.tasks.iter().map(|task| (client.remove(task), task));
    for (result, task) in results {
        match check_exists_cloud(task, client).await {
            Ok(true) => {}
            Ok(false) => return Err(CommandError::TaskNotFound { name: task.clone() }),
            Err(err) => return Err(err),
        }
        match result.await {
            Err(_) => return Err(CommandError::ConnectionError { name: task.clone() }),
            Ok(status_code) => {
                if !status_code.is_success() {
                    return Err(CommandError::HTTPError {
                        name: task.clone(),
                        status_code: status_code.as_u16(),
                    });
                }
            }
        }
    }

    Ok(())
}
