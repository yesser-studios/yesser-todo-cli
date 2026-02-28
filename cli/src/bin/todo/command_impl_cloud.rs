use std::{collections::HashSet, io::ErrorKind};

use yesser_todo_api::{Client, DEFAULT_PORT, api_error::ApiError};
use yesser_todo_db::SaveData;

use crate::{
    args::{ClearCommand, CloudCommand, TasksCommand},
    command_error::CommandError,
    utils::DONE_STYLE,
};

/// Checks whether a task with the given name exists on the cloud server.
///
/// Queries the cloud API for the task index; returns `true` when the server
/// returns a successful response and `false` when the server responds with
/// HTTP 404. Other HTTP status codes and request/connection failures are
/// returned as `CommandError` variants.
///
/// # Errors
///
/// Returns `CommandError::HTTPError { name, status_code }` for non-404 HTTP
/// responses and `CommandError::ConnectionError { name }` for request/connection
/// failures.
///
/// # Examples
///
/// ```ignore
/// // create or obtain a `Client` appropriate for your environment
/// let client = /* Client::new(...) */ ;
/// let exists = tokio::runtime::Runtime::new()
///     .unwrap()
///     .block_on(check_exists_cloud("my-task", &client))
///     .unwrap();
/// if exists {
///     println!("Task exists");
/// } else {
///     println!("Task not found");
/// }
/// ```
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

/// Adds one or more tasks to the cloud after validating input and existence.
///
/// Validates that `command.tasks` is non-empty, that task names are unique, and that none of the tasks already exist remotely; on success it sends add requests for each task and maps API errors to `CommandError`.
///
/// # Returns
///
/// `Ok(())` on success, `Err(CommandError)` if validation fails or an API/connection error occurs (possible variants include `NoTasksSpecified`, `DuplicateInput`, `TaskExists`, `HTTPError`, and `ConnectionError`).
///
/// # Examples
///
/// ```
/// // Illustrative example; adapt `command` and `client` to your test setup.
/// use futures::executor::block_on;
///
/// // let mut client = /* obtain Client */;
/// // let command = /* TasksCommand { tasks: vec!["task1".into()] } */;
///
/// // block_on(handle_add_cloud(&command, &mut client)).unwrap();
/// ```
pub(crate) async fn handle_add_cloud(command: &TasksCommand, client: &mut Client) -> Result<(), CommandError> {
    if command.tasks.len() <= 0 {
        return Err(CommandError::NoTasksSpecified);
    }

    let mut seen = HashSet::new();
    for task in &command.tasks {
        if !seen.insert(task.as_str()) {
            return Err(CommandError::DuplicateInput { name: task.clone() });
        }
        match check_exists_cloud(task, client).await {
            Ok(true) => return Err(CommandError::TaskExists { name: task.clone() }),
            Ok(false) => {}
            Err(err) => return Err(err),
        }
    }

    let results = command.tasks.iter().map(|task| (client.add(task), task));
    for (result, task) in results {
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

/// Removes the specified tasks from the cloud after validating input and confirming each task exists.
///
/// Validates that at least one task is provided and that task names are unique; verifies each task exists before attempting removal. On failure, returns the corresponding `CommandError`:
/// - `NoTasksSpecified` if no tasks were given.
/// - `DuplicateInput { name }` if the same task name appears more than once in the input.
/// - `TaskNotFound { name }` if a task does not exist on the server.
/// - `HTTPError { name, status_code }` for non-404 HTTP errors returned while removing a task.
/// - `ConnectionError { name }` for transport-level errors while communicating with the server.
///
/// # Examples
///
/// ```
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Assume `client` implements the cloud API client and `TasksCommand` wraps task names.
/// // let mut client = Client::new("http://api")?;
/// // let cmd = TasksCommand { tasks: vec!["buy milk".into()] };
/// // handle_remove_cloud(&cmd, &mut client).await?;
/// # Ok(()) }
/// ```
pub(crate) async fn handle_remove_cloud(command: &TasksCommand, client: &mut Client) -> Result<(), CommandError> {
    if command.tasks.len() <= 0 {
        return Err(CommandError::NoTasksSpecified);
    }

    let mut seen = HashSet::new();
    for task in &command.tasks {
        if !seen.insert(task.as_str()) {
            return Err(CommandError::DuplicateInput { name: task.clone() });
        }
        match check_exists_cloud(task, client).await {
            Ok(true) => {}
            Ok(false) => return Err(CommandError::TaskNotFound { name: task.clone() }),
            Err(err) => return Err(err),
        }
    }

    let results = command.tasks.iter().map(|task| (client.remove(task), task));
    for (result, task) in results {
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

/// List tasks from the cloud and print them to stdout.
///
/// Prints "Current tasks:" followed by each task name; completed tasks are styled with
/// `DONE_STYLE`. If the remote API responds with an HTTP error, returns `CommandError::HTTPError`
/// with an empty `name` and the response status code. If the request fails due to a connection
/// problem, returns `CommandError::ConnectionError` with an empty `name`.
///
/// # Examples
///
/// ```
/// # use todo_cli::cloud::Client;
/// # use todo_cli::command_impl_cloud::handle_list_cloud;
/// # tokio_test::block_on(async {
/// let client = Client::example(); // construct a client connected to a test server
/// let _ = handle_list_cloud(&client).await;
/// # });
/// ```
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

/// Marks the given tasks as done or undone in the cloud after validating input.
///
/// Validates that at least one task is provided and that task names are unique; returns
/// a `TaskNotFound` error if any task does not exist. For each task, sends the appropriate
/// request (`done` or `undone`) and maps HTTP and connection failures to `CommandError`.
///
/// # Examples
///
/// ```
/// # use todo_cli::{Client, TasksCommand, CommandError};
/// # async fn _example(mut client: Client) -> Result<(), CommandError> {
/// let cmd = TasksCommand { tasks: vec!["task1".into(), "task2".into()] };
/// // mark tasks as done
/// handle_done_undone_cloud(&cmd, &mut client, true).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Returns
///
/// `Ok(())` on success; `Err(CommandError)` on validation failures or API/connection errors.
pub(crate) async fn handle_done_undone_cloud(command: &TasksCommand, client: &mut Client, done: bool) -> Result<(), CommandError> {
    if command.tasks.len() <= 0 {
        return Err(CommandError::NoTasksSpecified);
    }

    let mut seen = HashSet::new();
    for task in &command.tasks {
        if !seen.insert(task.as_str()) {
            return Err(CommandError::DuplicateInput { name: task.clone() });
        }
        match check_exists_cloud(task, client).await {
            Ok(true) => {}
            Ok(false) => return Err(CommandError::TaskNotFound { name: task.clone() }),
            Err(err) => return Err(err),
        }
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

/// Clears tasks stored in the cloud according to the provided command.
///
/// When `command.done` is `true`, only completed tasks are removed; otherwise all tasks are removed.
/// On failure, returns a `CommandError` describing either an HTTP error (with an empty `name` and the HTTP status code)
/// or a connection error (with an empty `name`).
///
/// # Examples
///
/// ```no_run
/// # use todo_cli::command_impl_cloud::{handle_clear_cloud, ClearCommand, Client};
/// # async fn run_example(mut client: Client) -> Result<(), Box<dyn std::error::Error>> {
/// let cmd = ClearCommand { done: true };
/// handle_clear_cloud(&cmd, &mut client).await?;
/// # Ok(())
/// # }
/// ```
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

/// Clears completed tasks from the configured cloud backend and prints a deprecation notice.
///
/// This command is deprecated: it prints a short notice advising `clear -d` and then clears tasks
/// that are marked done on the remote server.
///
/// # Examples
///
/// ```no_run
/// # async fn example(mut client: Client) -> Result<(), CommandError> {
/// handle_clear_done_cloud(&mut client).await?;
/// # Ok(())
/// # }
/// ```
///
/// # Returns
///
/// `Ok(())` on success, `Err(CommandError)` if the operation fails.
#[deprecated]
pub(crate) async fn handle_clear_done_cloud(client: &mut Client) -> Result<(), CommandError> {
    println!("clear-done is deprecated. Use clear -d instead.");
    handle_clear_cloud(&ClearCommand { done: true }, client).await
}

/// Link the local client to a cloud server by saving the host and port to persistent config.
///
/// Saves the provided host and either the given port or the default port; on success prints a
/// confirmation message.
///
/// # Returns
///
/// `Ok(())` if the configuration was saved successfully, `CommandError::DataError` if saving fails.
///
/// # Examples
///
/// ```
/// use crate::CloudCommand;
/// use crate::command_impl_cloud::handle_connect;
///
/// let cmd = CloudCommand { host: "example.com".to_string(), port: None };
/// let _ = handle_connect(&cmd).unwrap();
/// ```
pub(crate) fn handle_connect(command: &CloudCommand) -> Result<(), CommandError> {
    let result = match &command.port {
        None => SaveData::save_cloud_config(&command.host, DEFAULT_PORT),
        Some(port) => SaveData::save_cloud_config(&command.host, port),
    };

    match result {
        Ok(()) => {
            println!("Successfully linked server.");
            Ok(())
        }
        Err(err) => Err(CommandError::DataError {
            what: "server configuration".to_string(),
            err,
        }),
    }
}

/// Unlinks the local configuration from the cloud server.
///
/// Removes the saved cloud configuration and prints a confirmation on success.
///
/// # Returns
///
/// `Ok(())` if the configuration was removed.
/// `Err(CommandError::UnlinkedError)` if no cloud configuration was found.
/// `Err(CommandError::DataError)` for other errors encountered while removing the configuration.
///
/// # Examples
///
/// ```
/// // Attempt to unlink; succeed when a configuration exists.
/// let _ = todo_cli::command_impl_cloud::handle_disconnect();
/// ```
pub(crate) fn handle_disconnect() -> Result<(), CommandError> {
    match SaveData::remove_cloud_config() {
        Ok(_) => {
            println!("Successfully unlinked server.");
            Ok(())
        }
        Err(err) => match err {
            yesser_todo_db::db_error::DatabaseError::IOError(io_err) if io_err.kind() == ErrorKind::NotFound => Err(CommandError::UnlinkedError),
            _ => Err(CommandError::DataError {
                what: format!("configuration"),
                err: err,
            }),
        },
    }
}
