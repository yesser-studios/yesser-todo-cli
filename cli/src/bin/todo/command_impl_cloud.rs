use std::io::ErrorKind;

use yesser_todo_api::{Client, api_error::ApiError};
use yesser_todo_db::SaveData;

use crate::{
    args::{ClearCommand, CloudCommand, TasksCommand},
    command_error::CommandError,
    utils::DONE_STYLE,
};

pub(crate) async fn check_exists_cloud(task: &str, client: &Client) -> Result<bool, CommandError> {
    let result = client.get_index(task).await;
    match result {
        Ok(_) => Ok(true),
        Err(err) => match err {
            ApiError::HTTPError(status_code) => {
                if status_code == 404 {
                    Ok(false)
                } else {
                    Err(CommandError::HTTPError {
                        name: task.to_string(),
                        status_code: status_code.as_u16(),
                    })
                }
            }
            ApiError::RequestError(_) => Err(CommandError::ConnectionError { name: task.to_string() }),
        },
    }
}

pub(crate) async fn handle_add_cloud(command: &TasksCommand, client: &mut Client) -> Result<(), CommandError> {
    if command.tasks.len() <= 0 {
        return Err(CommandError::NoTasksSpecified);
    }

    let results = command.tasks.iter().map(|task| (client.add(task), task));
    for (result, task) in results {
        match check_exists_cloud(task, client).await {
            Ok(true) => return Err(CommandError::TaskExists { name: task.clone() }),
            Ok(false) => {}
            Err(err) => return Err(err),
        }
        match result.await {
            Err(err) => match err {
                ApiError::HTTPError(status_code) => {
                    return Err(CommandError::HTTPError {
                        name: task.clone(),
                        status_code: status_code.as_u16(),
                    });
                }
                ApiError::RequestError(_) => return Err(CommandError::ConnectionError { name: task.clone() }),
            },
            Ok(_) => {}
        };
    }
    Ok(())
}

pub(crate) async fn handle_remove_cloud(command: &TasksCommand, client: &mut Client) -> Result<(), CommandError> {
    if command.tasks.len() <= 0 {
        return Err(CommandError::NoTasksSpecified);
    }

    let results = command.tasks.iter().map(|task| (client.remove(task), task));
    for (result, task) in results {
        match check_exists_cloud(task, client).await {
            Ok(true) => {}
            Ok(false) => return Err(CommandError::TaskNotFound { name: task.clone() }),
            Err(err) => return Err(err),
        }
        match result.await {
            Err(err) => match err {
                ApiError::HTTPError(status_code) => {
                    return Err(CommandError::HTTPError {
                        name: task.clone(),
                        status_code: status_code.as_u16(),
                    });
                }
                ApiError::RequestError(_) => return Err(CommandError::ConnectionError { name: task.clone() }),
            },
            Ok(_) => {}
        }
    }

    Ok(())
}

pub(crate) async fn handle_list_cloud(client: &Client) -> Result<(), CommandError> {
    let result = client.get().await;
    match result {
        Ok((_, tasks)) => {
            println!("\nCurrent tasks:");
            for task in tasks {
                if task.done {
                    println!("{}", DONE_STYLE.apply_to(&task.name))
                } else {
                    println!("{}", task.name)
                }
            }
        }
        Err(err) => match err {
            ApiError::HTTPError(status_code) => {
                return Err(CommandError::HTTPError {
                    name: "".to_string(),
                    status_code: status_code.as_u16(),
                });
            }
            ApiError::RequestError(_) => {
                return Err(CommandError::ConnectionError { name: "".to_string() });
            }
        },
    }

    Ok(())
}

pub(crate) async fn handle_done_undone_cloud(command: &TasksCommand, client: &mut Client, done: bool) -> Result<(), CommandError> {
    if command.tasks.len() <= 0 {
        return Err(CommandError::NoTasksSpecified);
    }

    for task in &command.tasks {
        let result = if done { client.done(task).await } else { client.undone(task).await };
        match result {
            Ok(_) => {}
            Err(err) => match err {
                ApiError::HTTPError(status_code) => {
                    if status_code.as_u16() == 404 {
                        return Err(CommandError::TaskNotFound { name: task.clone() });
                    } else {
                        return Err(CommandError::HTTPError {
                            name: task.clone(),
                            status_code: status_code.as_u16(),
                        });
                    }
                }

                ApiError::RequestError(_) => return Err(CommandError::ConnectionError { name: task.clone() }),
            },
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
        Ok(_) => Ok(()),
        Err(err) => match err {
            ApiError::HTTPError(status_code) => Err(CommandError::HTTPError {
                name: "".to_string(),
                status_code: status_code.as_u16(),
            }),
            ApiError::RequestError(_) => Err(CommandError::ConnectionError { name: "".to_string() }),
        },
    }
}

#[deprecated]
pub(crate) async fn handle_clear_done_cloud(client: &mut Client) -> Result<(), CommandError> {
    println!("clear-done is deprecated. Use clear -d instead.");
    handle_clear_cloud(&ClearCommand { done: true }, client).await
}

pub(crate) fn handle_connect(command: &CloudCommand) -> Result<(), CommandError> {
    let result = match &command.port {
        None => {
            let client = Client::new("".to_string(), None);
            SaveData::save_cloud_config(&command.host, &client.port)
        }
        Some(port) => SaveData::save_cloud_config(&command.host, port),
    };

    match result {
        Ok(()) => {
            println!("Successfully linked server.");
            Ok(())
        }
        Err(_) => Err(CommandError::DataError {
            what: "server configuration".to_string(),
        }),
    }
}

pub(crate) fn handle_disconnect() -> Result<(), CommandError> {
    match SaveData::remove_cloud_config() {
        Ok(_) => {
            println!("Successfully unlinked server.");
            Ok(())
        }
        Err(err) => match err.kind() {
            ErrorKind::NotFound => Err(CommandError::UnlinkedError),
            _ => Err(CommandError::DataError {
                what: format!("configuration: {err}"),
            }),
        },
    }
}
