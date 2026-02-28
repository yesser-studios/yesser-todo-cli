pub mod api_error;

use reqwest::StatusCode;
use yesser_todo_db::Task;

use crate::api_error::ApiError;

pub const DEFAULT_PORT: &str = "6982";

pub struct Client {
    pub hostname: String,
    pub port: String,
    client: reqwest::Client,
}

impl Client {
    /// Creates a new `Client` for the given hostname.
    ///
    /// If `port` is `None`, the client will use the default port `"6982"`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use yesser_todo_api::{Client, DEFAULT_PORT};
    ///
    /// let c = Client::new("http://127.0.0.1".to_string(), None);
    /// # assert_eq!(c.hostname, "http://127.0.0.1");
    /// # assert_eq!(c.port, DEFAULT_PORT);
    /// ```
    pub fn new(hostname: String, port: Option<String>) -> Client {
        match port {
            None => Client {
                hostname,
                port: DEFAULT_PORT.to_string(),
                client: reqwest::Client::new(),
            },
            Some(port) => Client {
                hostname,
                port,
                client: reqwest::Client::new(),
            },
        }
    }

    /// Retrieves all tasks from the configured server.
    ///
    /// # Returns
    ///
    /// A tuple `(StatusCode, Vec<Task>)` where `StatusCode` is the HTTP response status and `Vec<Task>` is the list of tasks.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use yesser_todo_api::Client;
    /// use yesser_todo_db::Task;
    ///
    /// # async fn example() -> Option<Vec<Task>> {
    /// let client = Client::new("http://127.0.0.1".into(), None);
    /// let (status, tasks) = client.get().await.ok()?;
    /// // `tasks` is a Vec<yesser_todo_db::Task>
    /// # Some(tasks)
    /// # }
    /// ```
    pub async fn get(&self) -> Result<(StatusCode, Vec<Task>), ApiError> {
        let result = self.client.get(format!("{}:{}/tasks", self.hostname, self.port).as_str()).send().await;

        match result {
            Ok(result) => {
                let status_code = result.status();
                if status_code.is_success() {
                    let result = result.json::<Vec<Task>>().await;
                    match result {
                        Ok(result) => Ok((status_code, result)),
                        Err(err) => Err(ApiError::RequestError(err)),
                    }
                } else {
                    Err(ApiError::HTTPError(status_code))
                }
            }
            Err(err) => Err(ApiError::RequestError(err)),
        }
    }

    /// Adds a new task with the given name to the to-do service.
    ///
    /// Sends the task name as JSON to the service's `/add` endpoint and returns the HTTP status
    /// together with the created `Task` parsed from the response.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use yesser_todo_api::Client;
    /// use reqwest::StatusCode;
    ///
    /// #[tokio::test]
    /// async fn example_add() {
    ///     let client = Client::new("http://127.0.0.1".to_string(), None);
    ///     let (status, task) = client.add("example task").await.unwrap();
    ///     assert!(status.is_success());
    ///     assert_eq!(task.name, "example task");
    /// }
    /// ```
    pub async fn add(&self, task_name: &str) -> Result<(StatusCode, Task), ApiError> {
        let result = self
            .client
            .post(format!("{}:{}/add", self.hostname, self.port).as_str())
            .json(&task_name)
            .send()
            .await;

        match result {
            Ok(result) => {
                let status_code = result.status();
                if status_code.is_success() {
                    let result = result.json::<Task>().await;
                    match result {
                        Ok(result) => Ok((status_code, result)),
                        Err(err) => Err(ApiError::RequestError(err)),
                    }
                } else {
                    Err(ApiError::HTTPError(status_code))
                }
            }
            Err(err) => Err(ApiError::RequestError(err)),
        }
    }

    /// Retrieves the zero-based index of the task with the given name from the API.
    ///
    /// Returns the HTTP response status together with the parsed index on success.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use yesser_todo_api::Client;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://127.0.0.1".into(), None);
    /// let (status, index) = client.get_index("example-task").await?;
    /// println!("status: {}, index: {}", status, index);
    /// # Ok(()) }
    /// ```
    pub async fn get_index(&self, task_name: &str) -> Result<(StatusCode, usize), ApiError> {
        let result = self
            .client
            .get(format!("{}:{}/index", self.hostname, self.port).as_str())
            .json(&task_name)
            .send()
            .await;
        match result {
            Ok(result) => {
                let status_code = result.status();
                if status_code.is_success() {
                    let result = result.json::<usize>().await;
                    match result {
                        Ok(result) => Ok((status_code, result)),
                        Err(err) => Err(ApiError::RequestError(err)),
                    }
                } else {
                    Err(ApiError::HTTPError(status_code))
                }
            }
            Err(err) => Err(ApiError::RequestError(err)),
        }
    }

    /// Delete the task with the given name from the remote server.
    ///
    /// Resolves the task's index on the server and sends a DELETE request for that index.
    ///
    /// # Returns
    ///
    /// `Ok(StatusCode)` if the server responded with a success status for the delete request, `Err(ApiError)` on transport or HTTP error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use yesser_todo_api::Client;
    /// # use tokio;
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::new("http://127.0.0.1".to_string(), None);
    ///     let _status = client.remove("example-task").await;
    /// }
    /// ```
    pub async fn remove(&self, task_name: &str) -> Result<StatusCode, ApiError> {
        let index_result = self.get_index(task_name).await;
        let index: usize;
        match index_result {
            Ok((_, result)) => {
                index = result;
            }
            Err(err) => return Err(err),
        }
        let result = self
            .client
            .delete(format!("{}:{}/remove", self.hostname, self.port).as_str())
            .json(&index)
            .send()
            .await;
        match result {
            Ok(result) => {
                if result.status().is_success() {
                    Ok(result.status())
                } else {
                    Err(ApiError::HTTPError(result.status()))
                }
            }
            Err(err) => Err(ApiError::RequestError(err)),
        }
    }

    /// Mark the task with the given name as done and return the server status and the updated task.
    ///
    /// On success returns a tuple containing the response `StatusCode` and the `Task` as returned by the server.
    /// On failure returns an `Err(ApiError)`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use yesser_todo_api::Client;
    /// # use yesser_todo_db::Task;
    /// # use reqwest::StatusCode;
    /// # async fn _example() {
    /// let client = Client::new("http://127.0.0.1".to_string(), None);
    /// let res = client.done("test").await;
    /// match res {
    ///     Ok((status, task)) => {
    ///         assert!(status.is_success() || status.is_client_error() || status.is_server_error());
    ///         let _ = task.name;
    ///     }
    ///     Err(e) => panic!("request failed: {:?}", e),
    /// }
    /// # }
    /// ```
    pub async fn done(&self, task_name: &str) -> Result<(StatusCode, Task), ApiError> {
        let index_result = self.get_index(task_name).await;
        let index: usize;
        match index_result {
            Ok((status_code, result)) => {
                if !status_code.is_success() {
                    return Err(ApiError::HTTPError(status_code));
                }
                index = result;
            }
            Err(err) => return Err(err),
        }
        let result = self
            .client
            .post(format!("{}:{}/done", self.hostname, self.port).as_str())
            .json(&index)
            .send()
            .await;
        match result {
            Ok(result) => {
                let status_code = result.status();
                if status_code.is_success() {
                    let result = result.json::<Task>().await;
                    match result {
                        Ok(result) => Ok((status_code, result)),
                        Err(err) => Err(ApiError::RequestError(err)),
                    }
                } else {
                    Err(ApiError::HTTPError(status_code))
                }
            }
            Err(err) => Err(ApiError::RequestError(err)),
        }
    }

    /// Mark the task identified by `task_name` as not done and return the updated task.
    ///
    /// # Returns
    ///
    /// `Ok((StatusCode, Task))` with the HTTP response status and the updated task on success, `Err(ApiError)` on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use yesser_todo_api::Client;
    /// use reqwest::StatusCode;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://127.0.0.1".to_string(), None);
    /// let res = client.undone("example").await?;
    /// assert!(matches!(res.0, StatusCode::OK) || res.0.is_client_error() || res.0.is_server_error());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn undone(&self, task_name: &str) -> Result<(StatusCode, Task), ApiError> {
        let index_result = self.get_index(task_name).await;
        let index: usize;
        match index_result {
            Ok((status_code, result)) => {
                if !status_code.is_success() {
                    return Err(ApiError::HTTPError(status_code));
                }
                index = result;
            }
            Err(err) => return Err(err),
        }
        let result = self
            .client
            .post(format!("{}:{}/undone", self.hostname, self.port).as_str())
            .json(&index)
            .send()
            .await;
        match result {
            Ok(result) => {
                let status_code = result.status();
                if status_code.is_success() {
                    let result = result.json::<Task>().await;
                    match result {
                        Ok(result) => Ok((status_code, result)),
                        Err(err) => Err(ApiError::RequestError(err)),
                    }
                } else {
                    Err(ApiError::HTTPError(status_code))
                }
            }
            Err(err) => Err(ApiError::RequestError(err)),
        }
    }

    /// Clears all tasks on the remote to-do service.
    ///
    /// Sends a DELETE request to the configured `/clear` endpoint and returns the HTTP status code.
    ///
    /// # Returns
    /// - `StatusCode` of the request;
    /// - on failure returns `Err(ApiError)`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use yesser_todo_api::Client;
    /// # async fn example() {
    /// let client = Client::new("http://127.0.0.1".to_string(), None);
    /// let status = client.clear().await.unwrap();
    /// assert_eq!(status, reqwest::StatusCode::OK);
    /// # }
    /// ```
    pub async fn clear(&self) -> Result<StatusCode, ApiError> {
        let result = self.client.delete(format!("{}:{}/clear", self.hostname, self.port).as_str()).send().await;
        match result {
            Ok(result) => {
                if result.status().is_success() {
                    Ok(result.status())
                } else {
                    Err(ApiError::HTTPError(result.status()))
                }
            }
            Err(err) => Err(ApiError::RequestError(err)),
        }
    }

    /// Clears all tasks marked as done on the remote to-do service.
    ///
    /// Sends a DELETE request to "{hostname}:{port}/cleardone".
    ///
    /// # Returns
    ///
    /// `Ok(StatusCode)` with the HTTP response status when the request succeeds, `Err(ApiError)` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use yesser_todo_api::Client;
    ///
    /// let client = Client::new("http://127.0.0.1".into(), None);
    /// let status = tokio::runtime::Runtime::new()
    ///     .unwrap()
    ///     .block_on(client.clear_done())
    ///     .unwrap();
    /// assert!(status.is_success());
    /// ```
    pub async fn clear_done(&self) -> Result<StatusCode, ApiError> {
        let result = self.client.delete(format!("{}:{}/cleardone", self.hostname, self.port).as_str()).send().await;
        match result {
            Ok(result) => {
                if result.status().is_success() {
                    Ok(result.status())
                } else {
                    Err(ApiError::HTTPError(result.status()))
                }
            }
            Err(err) => Err(ApiError::RequestError(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get() {
        let client = Client::new("http://127.0.0.1".to_string(), None);
        let result = client.get().await;
        println!("{:?}", result);
        assert!(result.is_ok() && result.unwrap().0.is_success());
    }

    #[tokio::test]
    async fn add_get_index_done_undone_remove() {
        let client = Client::new("http://127.0.0.1".to_string(), None);
        // add
        let result = client.add("test").await;
        println!("{:?}", result);
        assert!(result.is_ok() && result.unwrap().0.is_success());
        // get_index
        let result = client.get_index("test").await;
        println!("{:?}", result);
        assert!(result.is_ok() && result.unwrap().0.is_success());
        // done
        let result = client.done("test").await;
        println!("{:?}", result);
        assert!(result.is_ok() && result.unwrap().0.is_success());
        // undone
        let result = client.undone("test").await;
        println!("{:?}", result);
        assert!(result.is_ok() && result.unwrap().0.is_success());
        // remove
        let result = client.remove("test").await;
        println!("{:?}", result);
        assert!(result.is_ok() && result.unwrap().is_success());

        // cleanup
        let result = client.clear().await;
        println!("{:?}", result);
        assert!(result.is_ok() && result.unwrap().is_success());
    }

    /// Verifies that clearing removes all tasks and leaves the task list empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::Client;
    ///
    /// #[tokio::test]
    /// async fn clear() {
    ///     let client = Client::new("http://127.0.0.1".to_string(), None);
    ///     let _ = client.add("test").await;
    ///     let _ = client.add("test").await;
    ///     let _ = client.add("test").await;
    ///     let result = client.clear().await;
    ///     assert!(result.is_ok());
    ///     let result = client.get().await;
    ///     assert!(result.is_ok());
    ///     let unwrapped = result.unwrap();
    ///     assert!(unwrapped.0.is_success() && unwrapped.1.is_empty());
    /// }
    /// ```
    #[tokio::test]
    async fn clear() {
        let client = Client::new("http://127.0.0.1".to_string(), None);
        let _ = client.add("test").await;
        let _ = client.add("test").await;
        let _ = client.add("test").await;
        let result = client.clear().await;
        println!("{:?}", result);
        assert!(result.is_ok());
        let result = client.get().await;
        println!("{:?}", result);
        assert!(result.is_ok());
        let unwrapped = result.unwrap();
        assert!(unwrapped.0.is_success() && unwrapped.1.is_empty());
    }

    #[tokio::test]
    async fn clear_done() {
        let client = Client::new("http://127.0.0.1".to_string(), None);
        let _ = client.add("test1").await;
        let _ = client.add("test2").await;
        let _ = client.add("test3").await;
        let _ = client.done("test1").await;
        let _ = client.done("test3").await;
        let result = client.clear_done().await;
        println!("{:?}", result);
        assert!(result.is_ok());
        let result = client.get().await;
        println!("{:?}", result);
        assert!(result.is_ok());
        let unwrapped = result.unwrap();
        assert!(unwrapped.0.is_success() && unwrapped.1.len() == 1 && unwrapped.1[0].name == "test2");

        // cleanup
        let result = client.clear().await;
        println!("{:?}", result);
        assert!(result.is_ok() && result.unwrap().is_success());
    }
}
