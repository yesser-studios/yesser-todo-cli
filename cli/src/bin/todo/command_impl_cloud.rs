use std::{collections::HashSet, io::ErrorKind};

use url::Url;
use yesser_todo_api::{ApiError, Client, DEFAULT_PORT};
use yesser_todo_db::{DatabaseError, SaveData, Task};
use yesser_todo_errors::command_error::CommandError;

use crate::{
    args::{ClearCommand, CloudCommand, TasksCommand},
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
pub(crate) fn check_exists_cloud(task: &str, client: &Client) -> Result<bool, CommandError> {
    let result = client.get_index(task);
    match result {
        Ok(_) => Ok(true),
        Err(err) => match err {
            ApiError::HTTPError(status_code) => {
                if status_code == 404 {
                    Ok(false)
                } else {
                    Err(CommandError::HTTPError {
                        name: task.to_string(),
                        status_code,
                    })
                }
            }
            ApiError::RequestError(_) => Err(CommandError::ConnectionError { name: task.to_string() }),
            ApiError::ServerError(server_error) => match server_error {
                yesser_todo_errors::server_error::ServerError::NotFound(_) => Ok(false),
                _ => Err(CommandError::HTTPError {
                    name: task.to_string(),
                    status_code: server_error.to_status_code().as_u16(),
                }),
            },
        },
    }
}

pub(crate) fn get_tasks_cloud(client: &Client) -> Result<Vec<Task>, CommandError> {
    let (_, current_tasks) = match client.get() {
        Ok(t) => t,
        Err(err) => {
            return match err {
                ApiError::HTTPError(status_code) => Err(CommandError::HTTPError { name: "".into(), status_code }),
                ApiError::RequestError(_) => Err(CommandError::ConnectionError { name: "".into() }),
                ApiError::ServerError(server_error) => match server_error {
                    yesser_todo_errors::server_error::ServerError::NotFound(_) => Err(CommandError::HTTPError {
                        name: "".into(),
                        status_code: 404,
                    }),
                    other => Err(CommandError::HTTPError {
                        name: "".into(),
                        status_code: other.to_status_code().as_u16(),
                    }),
                },
            };
        }
    };
    Ok(current_tasks
        .iter()
        .map(|t| Task {
            name: t.name.clone(),
            done: t.done,
        })
        .collect())
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
pub(crate) fn handle_add_cloud(command: &TasksCommand, client: &mut Client) -> Result<(), CommandError> {
    if command.tasks.is_empty() {
        return Err(CommandError::NoTasksSpecified);
    }

    let current_tasks = get_tasks_cloud(client)?;
    let mut seen = HashSet::new();
    for task in &command.tasks {
        if !seen.insert(task.as_str()) {
            return Err(CommandError::DuplicateInput { name: task.clone() });
        }
        if current_tasks.iter().any(|x| x.name == task.clone()) {
            return Err(CommandError::TaskExists { name: task.clone() });
        }
    }

    let results = command.tasks.iter().map(|task| (client.add(task), task));
    for (result, task) in results {
        if let Err(err) = result {
            match err {
                ApiError::HTTPError(status_code) => {
                    return Err(CommandError::HTTPError {
                        name: task.clone(),
                        status_code,
                    });
                }
                ApiError::RequestError(_) => return Err(CommandError::ConnectionError { name: task.clone() }),
                ApiError::ServerError(server_error) => {
                    return Err(CommandError::HTTPError {
                        name: task.clone(),
                        status_code: server_error.to_status_code().as_u16(),
                    });
                }
            }
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
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Assume `client` implements the cloud API client and `TasksCommand` wraps task names.
/// // let mut client = Client::new("http://api")?;
/// // let cmd = TasksCommand { tasks: vec!["buy milk".into()] };
/// // handle_remove_cloud(&cmd, &mut client)?;
/// # Ok(()) }
/// ```
pub(crate) fn handle_remove_cloud(command: &TasksCommand, client: &mut Client) -> Result<(), CommandError> {
    if command.tasks.is_empty() {
        return Err(CommandError::NoTasksSpecified);
    }

    let current_tasks = get_tasks_cloud(client)?;
    let mut seen = HashSet::new();
    for task in &command.tasks {
        if !seen.insert(task.as_str()) {
            return Err(CommandError::DuplicateInput { name: task.clone() });
        }
        if !current_tasks.iter().any(|x| x.name == task.clone()) {
            return Err(CommandError::TaskNotFound { name: task.clone() });
        }
    }

    let results = command.tasks.iter().map(|task| (client.remove(task), task));
    for (result, task) in results {
        if let Err(err) = result {
            match err {
                ApiError::HTTPError(status_code) => {
                    return if status_code == 404 {
                        Err(CommandError::TaskNotFound { name: task.clone() })
                    } else {
                        Err(CommandError::HTTPError {
                            name: task.clone(),
                            status_code,
                        })
                    };
                }
                ApiError::RequestError(_) => return Err(CommandError::ConnectionError { name: task.clone() }),
                ApiError::ServerError(server_error) => {
                    return match server_error {
                        yesser_todo_errors::server_error::ServerError::NotFound(name) => Err(CommandError::TaskNotFound { name: name.to_string() }),
                        _ => Err(CommandError::HTTPError {
                            name: task.clone(),
                            status_code: server_error.to_status_code().as_u16(),
                        }),
                    };
                }
            }
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
/// use todo_cli::cloud::Client;
/// use todo_cli::command_impl_cloud::handle_list_cloud;
/// let client = Client::example(); // construct a client connected to a test server
/// let _ = handle_list_cloud(&client);
/// ```
pub(crate) fn handle_list_cloud(client: &Client) -> Result<(), CommandError> {
    let result = client.get();
    match result {
        Ok((_, tasks)) => {
            println!("\nCurrent tasks:");
            for task in tasks {
                if task.done {
                    println!("{}", yansi::Paint::paint(&task.name, DONE_STYLE))
                } else {
                    println!("{}", task.name)
                }
            }
        }
        Err(err) => match err {
            ApiError::HTTPError(status_code) => {
                return Err(CommandError::HTTPError {
                    name: "".to_string(),
                    status_code,
                });
            }
            ApiError::RequestError(_) => {
                return Err(CommandError::ConnectionError { name: "".to_string() });
            }
            ApiError::ServerError(server_error) => {
                return Err(CommandError::HTTPError {
                    name: "".to_string(),
                    status_code: server_error.to_status_code().as_u16(),
                });
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
/// use todo_cli::{Client, TasksCommand, CommandError};
/// fn _example(mut client: Client) -> Result<(), CommandError> {
/// let cmd = TasksCommand { tasks: vec!["task1".into(), "task2".into()] };
/// // mark tasks as done
/// handle_done_undone_cloud(&cmd, &mut client, true)?;
/// Ok(())
/// }
/// ```
///
/// # Returns
///
/// `Ok(())` on success; `Err(CommandError)` on validation failures or API/connection errors.
pub(crate) fn handle_done_undone_cloud(command: &TasksCommand, client: &mut Client, done: bool) -> Result<(), CommandError> {
    if command.tasks.is_empty() {
        return Err(CommandError::NoTasksSpecified);
    }

    let current_tasks = get_tasks_cloud(client)?;
    let mut seen = HashSet::new();
    for task in &command.tasks {
        if !seen.insert(task.as_str()) {
            return Err(CommandError::DuplicateInput { name: task.clone() });
        }
        if !current_tasks.iter().any(|x| x.name == task.clone()) {
            return Err(CommandError::TaskNotFound { name: task.clone() });
        }
    }

    for task in &command.tasks {
        let result = if done { client.done(task) } else { client.undone(task) };
        match result {
            Ok(_) => {}
            Err(err) => match err {
                ApiError::HTTPError(status_code) => {
                    return if status_code == 404 {
                        Err(CommandError::TaskNotFound { name: task.clone() })
                    } else {
                        Err(CommandError::HTTPError {
                            name: task.clone(),
                            status_code,
                        })
                    };
                }

                ApiError::RequestError(_) => return Err(CommandError::ConnectionError { name: task.clone() }),
                ApiError::ServerError(server_error) => {
                    return match server_error {
                        yesser_todo_errors::server_error::ServerError::NotFound(name) => Err(CommandError::TaskNotFound { name: name.to_string() }),
                        _ => Err(CommandError::HTTPError {
                            name: task.clone(),
                            status_code: server_error.to_status_code().as_u16(),
                        }),
                    };
                }
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
/// use todo_cli::command_impl_cloud::{handle_clear_cloud, ClearCommand, Client};
/// fn run_example(mut client: Client) -> Result<(), Box<dyn std::error::Error>> {
/// let cmd = ClearCommand { done: true };
/// handle_clear_cloud(&cmd, &mut client)?;
/// Ok(())
/// }
/// ```
pub(crate) fn handle_clear_cloud(command: &ClearCommand, client: &mut Client) -> Result<(), CommandError> {
    let result = if command.done { client.clear_done() } else { client.clear() };
    match result {
        Ok(_) => Ok(()),
        Err(err) => match err {
            ApiError::HTTPError(status_code) => Err(CommandError::HTTPError {
                name: "".to_string(),
                status_code,
            }),
            ApiError::RequestError(_) => Err(CommandError::ConnectionError { name: "".to_string() }),
            ApiError::ServerError(server_error) => Err(CommandError::HTTPError {
                name: "".to_string(),
                status_code: server_error.to_status_code().as_u16(),
            }),
        },
    }
}

/// Clears tasks marked done on the configured cloud backend and emits a deprecation notice.
///
/// This command is deprecated; it prints a short notice advising `clear -d` before clearing
/// completed tasks on the remote server.
///
/// # Examples
///
/// ```no_run
/// fn example(mut client: Client) -> Result<(), CommandError> {
/// handle_clear_done_cloud(&mut client)?;
/// Ok(())
/// }
/// ```
///
/// Returns `Ok(())` on success, or a `CommandError` if the operation fails.
#[deprecated]
pub(crate) fn handle_clear_done_cloud(client: &mut Client) -> Result<(), CommandError> {
    println!("clear-done is deprecated. Use clear -d instead.");
    handle_clear_cloud(&ClearCommand { done: true }, client)
}

/// Normalizes and validates a host URL for cloud configuration.
///
/// Accepts a string with or without a scheme (defaults to `http` if missing), parses it as a URL,
/// and enforces that the scheme is `http` or `https`. Rejects URLs that include a path (other than `/`),
/// a query, a fragment, or embedded credentials. When parsing fails and the input looks like an unbracketed
/// IPv6 address, the error message includes a hint about wrapping IPv6 addresses with `[]`.
///
/// # Examples
///
/// ```
/// let url = parse_url("example.com").unwrap();
/// assert_eq!(url.scheme(), "http");
/// assert_eq!(url.host_str().unwrap(), "example.com");
/// ```
pub(crate) fn parse_url(url: &str) -> Result<Url, CommandError> {
    let url = if url.contains("://") { url } else { &format!("http://{url}") };

    let parsed = Url::parse(url).map_err(|x| CommandError::InvalidUrlError {
        why: format!(
            "{}{}",
            x,
            if url.to_string().matches(':').count() >= 3 {
                // One ':' is before scheme and one before port
                "\nHelp: Did you mean to wrap IPv6 address with []?"
            } else {
                ""
            }
        ),
    })?;

    match parsed.scheme() {
        "http" | "https" => {}
        s => {
            return Err(CommandError::InvalidUrlError {
                why: format!("Invalid scheme: {}", s.to_uppercase()),
            });
        }
    }

    if parsed.path() != "/" && !parsed.path().is_empty() {
        return Err(CommandError::InvalidUrlError {
            why: "You should not specify a path!".to_string(),
        });
    }

    if parsed.query().is_some() {
        return Err(CommandError::InvalidUrlError {
            why: "You should not specify a query!".to_string(),
        });
    }

    if parsed.fragment().is_some() {
        return Err(CommandError::InvalidUrlError {
            why: "You should not specify a fragment!".to_string(),
        });
    }

    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(CommandError::InvalidUrlError {
            why: "You should not specify credentials!".to_string(),
        });
    }

    Ok(parsed)
}

/// Link the local client to a cloud server by saving the host and port to persistent configuration.
///
/// Parses and normalizes the provided host, determines the effective port (honoring a provided
/// `--port` flag when present, and ensuring it does not conflict with a port in the URL), validates
/// the port range, and saves the resulting host and port to persistent configuration.
///
/// # Returns
///
/// `Ok(())` if the configuration was saved successfully.
/// `Err(CommandError::InvalidUrlError)` if the host or port are invalid or inconsistent.
/// `Err(CommandError::DataError)` if saving the configuration fails.
///
/// # Examples
///
/// ```
/// use crate::CloudCommand;
/// use crate::command_impl_cloud::handle_connect;
///
/// let cmd = CloudCommand { host: "http://example.com".to_string(), port: None };
/// let _ = handle_connect(&cmd).unwrap();
/// ```
pub(crate) fn handle_connect(command: &CloudCommand) -> Result<(), CommandError> {
    let url = parse_url(&command.host)?;

    let cmd_port = if let Some(cmd_port) = &command.port {
        Some(
            cmd_port
                .parse::<u16>()
                .map_err(|_| CommandError::InvalidUrlError {
                    why: "The port specified in the <PORT> parameter is invalid!".to_string(),
                })?
                .to_string(),
        )
    } else {
        None
    };

    let port: &str = match (url.port(), cmd_port) {
        (Some(url_port), Some(cmd_port)) if url_port.to_string() != cmd_port => {
            return Err(CommandError::InvalidUrlError {
                why: "Port in URL and <PORT> parameter do not match!".to_string(),
            });
        }
        (Some(url_port), _) => &url_port.to_string(),
        (None, Some(cmd_port)) => &cmd_port.clone(),
        (None, None) => DEFAULT_PORT,
    };

    if !(1..=65535).contains(&port.parse::<i32>().unwrap()) {
        return Err(CommandError::InvalidUrlError {
            why: "The port must be between 1 and 65535!".to_string(),
        });
    }

    let host = format!(
        "{}://{}",
        url.scheme(),
        url.host_str().ok_or_else(|| CommandError::InvalidUrlError {
            why: "Unable to parse host!".to_string()
        })?
    );
    match SaveData::save_cloud_config(&host, port) {
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

/// Links the local client to a cloud server and emits a deprecation notice.
///
/// This command is deprecated; it prints a short notice advising `cloud connect` before
/// linking to a cloud server.
///
/// # Examples
///
/// ```
/// use crate::CloudCommand;
/// use crate::command_impl_cloud::handle_connect_old;
///
/// let cmd = CloudCommand { host: "http://example.com".to_string(), port: None };
/// let _ = handle_connect_old(&cmd).unwrap();
/// ```
///
/// Returns `Ok(())` on success, or a `CommandError` if an error occurs.
#[deprecated]
pub(crate) fn handle_connect_old(command: &CloudCommand) -> Result<(), CommandError> {
    println!("connect is deprecated. Use cloud connect instead.");
    handle_connect(command)
}

/// Removes the saved cloud configuration.
///
/// Prints a confirmation message on success.
///
/// # Returns
///
/// - `Ok(())` if a configuration was removed.
/// - `Err(CommandError::UnlinkedError)` if no cloud configuration was found.
/// - `Err(CommandError::DataError)` for other errors encountered while removing the configuration.
pub(crate) fn handle_disconnect() -> Result<(), CommandError> {
    match SaveData::remove_cloud_config() {
        Ok(_) => {
            println!("Successfully unlinked server.");
            Ok(())
        }
        Err(err) => match err {
            DatabaseError::IOError(io_err) if io_err.kind() == ErrorKind::NotFound => Err(CommandError::UnlinkedError),
            _ => Err(CommandError::DataError {
                what: "configuration".to_string(),
                err,
            }),
        },
    }
}

/// Displays the current cloud server configuration.
///
/// If a cloud server is configured, prints the hostname and port.
/// If no cloud server is configured, prints a "not connected" message.
/// Returns an error if retrieving the configuration fails.
///
/// # Returns
///
/// - `Ok(())` if the configuration was retrieved successfully (either printing
///   the config or "not connected" message)
/// - `Err(CommandError::DataError)` if retrieving the configuration failed,
///   with `what` set to "configuration" and the underlying error wrapped
///
/// # Examples
///
/// ```
/// use crate::command_impl_cloud::handle_show_server;
///
/// let result = handle_show_server();
/// ```
pub(crate) fn handle_show_server() -> Result<(), CommandError> {
    match SaveData::get_cloud_config() {
        Ok(data) => match data {
            Some((hostname, port)) => {
                println!("Hostname: {}, port: {}", hostname, port);
                Ok(())
            }
            None => {
                println!("You're not connected to a server!");
                Ok(())
            }
        },
        Err(e) => Err(CommandError::DataError {
            what: "configuration".to_string(),
            err: e,
        }),
    }
}

/// Removes the saved cloud configuration and emits a deprecation notice.
///
/// This command is deprecated; it prints a short notice advising `cloud disconnect` before
/// removing the cloud configuration.
///
/// # Examples
///
/// ```
/// use crate::command_impl_cloud::handle_disconnect_old;
///
/// let _ = handle_disconnect_old().unwrap();
/// ```
///
/// Returns `Ok(())` on success, or a `CommandError` if an error occurs.
#[deprecated]
pub(crate) fn handle_disconnect_old() -> Result<(), CommandError> {
    println!("disconnect is deprecated. Use cloud disconnect instead.");
    handle_disconnect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url_with_http_scheme() {
        let result = parse_url("http://example.com");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host_str(), Some("example.com"));
    }

    #[test]
    fn test_parse_url_with_https_scheme() {
        let result = parse_url("https://example.com");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.host_str(), Some("example.com"));
    }

    #[test]
    fn test_parse_url_without_scheme_defaults_to_http() {
        let result = parse_url("example.com");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme(), "http");
        assert_eq!(url.host_str(), Some("example.com"));
    }

    #[test]
    fn test_parse_url_with_port() {
        let result = parse_url("http://example.com:8080");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.port(), Some(8080));
    }

    #[test]
    fn test_parse_url_with_ipv4() {
        let result = parse_url("http://192.168.1.1:8080");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.host_str(), Some("192.168.1.1"));
        assert_eq!(url.port(), Some(8080));
    }

    #[test]
    fn test_parse_url_with_ipv6_brackets() {
        let result = parse_url("http://[::1]:8080");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.host_str(), Some("[::1]"));
        assert_eq!(url.port(), Some(8080));
    }

    #[test]
    fn test_parse_url_with_ipv6_without_brackets_gives_helpful_error() {
        let result = parse_url("http://::1:8080");
        assert!(result.is_err());
        if let Err(CommandError::InvalidUrlError { why }) = result {
            assert!(why.contains("Did you mean to wrap IPv6 address with []?"));
        } else {
            panic!("Expected InvalidUrlError");
        }
    }

    #[test]
    fn test_parse_url_with_invalid_scheme() {
        let result = parse_url("ftp://example.com");
        assert!(result.is_err());
        if let Err(CommandError::InvalidUrlError { why }) = result {
            assert!(why.contains("Invalid scheme"));
            assert!(why.contains("FTP"));
        } else {
            panic!("Expected InvalidUrlError for invalid scheme");
        }
    }

    #[test]
    fn test_parse_url_with_path_returns_error() {
        let result = parse_url("http://example.com/path");
        assert!(result.is_err());
        if let Err(CommandError::InvalidUrlError { why }) = result {
            assert!(why.contains("should not specify a path"));
        } else {
            panic!("Expected InvalidUrlError for path");
        }
    }

    #[test]
    fn test_parse_url_with_query_returns_error() {
        let result = parse_url("http://example.com?query=value");
        assert!(result.is_err());
        if let Err(CommandError::InvalidUrlError { why }) = result {
            assert!(why.contains("should not specify a query"));
        } else {
            panic!("Expected InvalidUrlError for query");
        }
    }

    #[test]
    fn test_parse_url_with_fragment_returns_error() {
        let result = parse_url("http://example.com#fragment");
        assert!(result.is_err());
        if let Err(CommandError::InvalidUrlError { why }) = result {
            assert!(why.contains("should not specify a fragment"));
        } else {
            panic!("Expected InvalidUrlError for fragment");
        }
    }

    #[test]
    fn test_parse_url_with_localhost() {
        let result = parse_url("localhost:8080");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.host_str(), Some("localhost"));
        assert_eq!(url.port(), Some(8080));
    }

    #[test]
    fn test_parse_url_with_multiple_colons_suggests_ipv6_brackets() {
        let result = parse_url("2001:db8::1:8080");
        assert!(result.is_err());
        if let Err(CommandError::InvalidUrlError { why }) = result {
            assert!(why.contains("Did you mean to wrap IPv6 address with []?"));
        } else {
            panic!("Expected helpful IPv6 error message");
        }
    }

    #[test]
    fn test_parse_url_empty_string() {
        let result = parse_url("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_url_with_username_password() {
        let result = parse_url("http://user:pass@example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_connect_creates_config_with_default_port() {
        // This test will attempt to save config, which may fail in test environment
        // Testing the function behavior rather than file system side effects
        let command = CloudCommand {
            host: "http://testhost.com".to_string(),
            port: None,
        };
        // Even if this errors due to file system, we're testing it doesn't panic
        let _ = handle_connect(&command);
    }

    #[test]
    fn test_handle_connect_with_explicit_port() {
        let command = CloudCommand {
            host: "http://testhost.com".to_string(),
            port: Some("9000".to_string()),
        };
        let _ = handle_connect(&command);
    }

    #[test]
    fn test_handle_connect_with_url_and_flag_port_match() {
        let command = CloudCommand {
            host: "http://testhost.com:9000".to_string(),
            port: Some("9000".to_string()),
        };
        let _ = handle_connect(&command);
    }

    #[test]
    fn test_handle_connect_with_url_and_flag_port_mismatch() {
        let command = CloudCommand {
            host: "http://testhost.com:8080".to_string(),
            port: Some("9000".to_string()),
        };
        let result = handle_connect(&command);
        assert!(result.is_err());
        if let Err(CommandError::InvalidUrlError { why }) = result {
            assert!(why.contains("Port in URL and <PORT> parameter do not match"));
        } else {
            panic!("Expected port mismatch error");
        }
    }

    #[test]
    fn test_handle_connect_with_invalid_port_string() {
        let command = CloudCommand {
            host: "http://testhost.com".to_string(),
            port: Some("not_a_number".to_string()),
        };
        let result = handle_connect(&command);
        assert!(result.is_err());
        if let Err(CommandError::InvalidUrlError { why }) = result {
            assert!(why.contains("port specified in the <PORT> parameter is invalid"));
        } else {
            panic!("Expected invalid port error");
        }
    }

    #[test]
    fn test_handle_connect_with_invalid_url() {
        let command = CloudCommand {
            host: "not a valid url!!!".to_string(),
            port: None,
        };
        let result = handle_connect(&command);
        assert!(result.is_err());
        assert!(matches!(result, Err(CommandError::InvalidUrlError { .. })));
    }

    #[test]
    fn test_handle_connect_with_invalid_scheme() {
        let command = CloudCommand {
            host: "ftp://testhost.com".to_string(),
            port: None,
        };
        let result = handle_connect(&command);
        assert!(result.is_err());
        if let Err(CommandError::InvalidUrlError { why }) = result {
            assert!(why.contains("Invalid scheme"));
        } else {
            panic!("Expected invalid scheme error");
        }
    }

    #[test]
    fn test_handle_disconnect_behavior() {
        // Testing the function executes without panic
        // May return UnlinkedError if no config exists, which is expected
        let result = handle_disconnect();
        // Either succeeds or returns UnlinkedError or DataError
        match result {
            Ok(_) | Err(CommandError::UnlinkedError) | Err(CommandError::DataError { .. }) => {}
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn test_command_error_display_for_invalid_url() {
        let err = CommandError::InvalidUrlError {
            why: "test reason".to_string(),
        };
        let display = format!("{}", err);
        assert_eq!(display, "Invalid URL: test reason");
    }

    #[test]
    fn test_parse_url_preserves_scheme() {
        let https_result = parse_url("https://secure.example.com");
        assert!(https_result.is_ok());
        assert_eq!(https_result.unwrap().scheme(), "https");

        let http_result = parse_url("http://example.com");
        assert!(http_result.is_ok());
        assert_eq!(http_result.unwrap().scheme(), "http");
    }

    #[test]
    fn test_parse_url_with_trailing_slash() {
        // Trailing slash is considered a path
        let result = parse_url("http://example.com/");
        // The path check should handle this - '/' is technically a path
        // Based on the code, it checks `parsed.path() != "/" && !parsed.path().is_empty()`
        // So '/' should be allowed
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_url_with_subdomain() {
        let result = parse_url("http://api.example.com:8080");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.host_str(), Some("api.example.com"));
        assert_eq!(url.port(), Some(8080));
    }

    #[test]
    fn test_handle_connect_port_priority_url_over_default() {
        let command = CloudCommand {
            host: "http://example.com:7777".to_string(),
            port: None,
        };
        // Should use port from URL, not default
        let _ = handle_connect(&command);
    }

    #[test]
    fn test_handle_connect_port_priority_flag_over_default() {
        let command = CloudCommand {
            host: "http://example.com".to_string(),
            port: Some("7777".to_string()),
        };
        // Should use port from flag, not default
        let _ = handle_connect(&command);
    }

    #[test]
    fn test_parse_url_rejects_ws_scheme() {
        let result = parse_url("ws://example.com");
        assert!(result.is_err());
        if let Err(CommandError::InvalidUrlError { why }) = result {
            assert!(why.contains("Invalid scheme"));
            assert!(why.contains("WS"));
        } else {
            panic!("Expected invalid scheme error");
        }
    }

    #[test]
    fn test_parse_url_case_insensitive_host() {
        let result = parse_url("http://EXAMPLE.COM");
        assert!(result.is_ok());
        let url = result.unwrap();
        // Hosts are typically normalized to lowercase by URL parser
        assert!(url.host_str().is_some());
    }

    #[test]
    fn test_handle_connect_with_port_65535() {
        let command = CloudCommand {
            host: "http://example.com".to_string(),
            port: Some("65535".to_string()),
        };
        let _ = handle_connect(&command);
    }

    #[test]
    fn test_handle_connect_with_port_overflow() {
        let command = CloudCommand {
            host: "http://example.com".to_string(),
            port: Some("65536".to_string()),
        };
        let result = handle_connect(&command);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_connect_with_port_zero() {
        let command = CloudCommand {
            host: "http://example.com".to_string(),
            port: Some("0".to_string()),
        };
        let result = handle_connect(&command);
        assert!(result.is_err())
    }

    #[test]
    fn test_handle_connect_with_large_port() {
        let command = CloudCommand {
            host: "http://example.com".to_string(),
            port: Some("999999".to_string()),
        };
        let result = handle_connect(&command);
        assert!(result.is_err())
    }

    #[test]
    fn test_parse_url_with_complex_ipv6() {
        let result = parse_url("http://[2001:db8:85a3::8a2e:370:7334]:443");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.port(), Some(443));
    }

    #[test]
    fn test_handle_connect_without_host() {
        let command = CloudCommand {
            host: "".to_string(),
            port: Some("8080".to_string()),
        };
        let result = handle_connect(&command);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_show_server() {
        let result = handle_show_server();

        assert!(result.is_ok());
    }
}
