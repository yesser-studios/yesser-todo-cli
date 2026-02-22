use yesser_todo_api::Client;

use crate::{
    args::{ClearCommand, TasksCommand},
    command_error::CommandError,
    utils::DONE_STYLE,
};

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
        Err(_) => Err(CommandError::ConnectionError { name: task.to_string() }),
    }
}

pub(crate) async fn handle_add_cloud(command: &TasksCommand, client: &mut Client) -> Result<(), CommandError> {
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

pub(crate) async fn handle_remove_cloud(command: &TasksCommand, client: &mut Client) -> Result<(), CommandError> {
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

pub(crate) async fn handle_list_cloud(client: &Client) -> Result<(), CommandError> {
    let result = client.get().await;
    match result {
        Ok((status_code, tasks)) => {
            if status_code.is_success() {
                println!("\nCurrent tasks:");
                for task in tasks {
                    if task.done {
                        println!("{}", DONE_STYLE.apply_to(&task.name))
                    } else {
                        println!("{}", task.name)
                    }
                }
            } else {
                return Err(CommandError::HTTPError {
                    name: "".to_string(),
                    status_code: status_code.as_u16(),
                });
            }
        }
        Err(_) => {
            return Err(CommandError::ConnectionError { name: "".to_string() });
        }
    }

    Ok(())
}

pub(crate) async fn handle_done_undone_cloud(command: &TasksCommand, client: &mut Client, done: bool) -> Result<(), CommandError> {
    for task in &command.tasks {
        let result = if done { client.done(task).await } else { client.undone(task).await };
        match result {
            Ok((status_code, _)) => {
                if status_code.is_success() {
                } else if status_code.as_u16() == 404 {
                    return Err(CommandError::TaskNotFound { name: task.clone() });
                } else {
                    return Err(CommandError::HTTPError {
                        name: task.clone(),
                        status_code: status_code.as_u16(),
                    });
                }
            }
            Err(_) => return Err(CommandError::ConnectionError { name: task.clone() }),
        }
    }
    Ok(())
}

pub(crate) async fn handle_clear_cloud(command: &ClearCommand, client: &mut Client) -> Result<(), CommandError> {
    let result;
    if command.done {
        result = client.clear_done().await;
    } else {
        result = client.clear().await;
    }
    match result {
        Ok(status_code) => {
            if status_code.is_success() {
                Ok(())
            } else {
                Err(CommandError::HTTPError {
                    name: "".to_string(),
                    status_code: status_code.as_u16(),
                })
            }
        }
        Err(_) => Err(CommandError::ConnectionError { name: "".to_string() }),
    }
}

#[deprecated]
pub(crate) async fn handle_clear_done_cloud(client: &mut Client) -> Result<(), CommandError> {
    println!("clear-done is deprecated. Use clear -d instead.");
    handle_clear_cloud(&ClearCommand { done: true }, client).await
}
