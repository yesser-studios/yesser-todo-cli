use reqwest::{Error, StatusCode};
use std::string::ToString;
use yesser_todo_db::Task;

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
    /// let c = Client::new("http://127.0.0.1".to_string(), None);
    /// assert_eq!(c.hostname, "http://127.0.0.1");
    /// assert_eq!(c.port, "6982");
    /// ```
    pub fn new(hostname: &str, port: Option<String>) -> Client {
        match port {
            None => Client {
                hostname: hostname.to_string(),
                port: "6982".to_string(),
                client: reqwest::Client::new(),
            },
            Some(port) => Client {
                hostname: hostname.to_string(),
                port,
                client: reqwest::Client::new(),
            },
        }
    }

    /// Fetches all tasks from the configured server.
    ///
    /// # Returns
    ///
    /// `(StatusCode, Vec<Task>)` where the `StatusCode` is the HTTP response status and the `Vec<Task>` is the list of tasks parsed from the response body.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use api::Client;
    /// use reqwest::StatusCode;
    ///
    /// let client = Client::new("http://127.0.0.1".into(), None);
    /// let rt = tokio::runtime::Runtime::new().unwrap();
    /// let (status, tasks) = rt.block_on(client.get()).unwrap();
    /// assert!(status == StatusCode::OK || status.is_success());
    /// // `tasks` is a Vec<yesser_todo_db::Task>
    /// ```
    pub async fn get(&self) -> Result<(StatusCode, Vec<Task>), Error> {
        let result = self.client.get(format!("{}:{}/tasks", self.hostname, self.port).as_str()).send().await;

        match result {
            Ok(result) => {
                let status_code = result.status();
                let result = result.json::<Vec<Task>>().await;
                match result {
                    Ok(result) => Ok((status_code, result)),
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(err),
        }
    }

    /// Adds a new task with the given name to the to-do service.
    ///
    /// Sends the task name as JSON to the service's `/add` endpoint and returns the HTTP status
    /// together with the created `Task` parsed from the response.
    ///
    /// # Parameters
    ///
    /// - `task_name`: The name of the task to create.
    ///
    /// # Returns
    ///
    /// A `(StatusCode, Task)` tuple containing the HTTP response status and the created `Task`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use yesser_todo_api::Client;
    /// # use reqwest::StatusCode;
    /// # #[tokio::test]
    /// # async fn example_add() {
    /// let client = Client::new("http://127.0.0.1".to_string(), None);
    /// let (status, task) = client.add(&"example task".to_string()).await.unwrap();
    /// assert_eq!(status, StatusCode::OK);
    /// assert_eq!(task.name, "example task");
    /// # }
    /// ```
    pub async fn add(&self, task_name: &str) -> Result<(StatusCode, Task), Error> {
        let result = self
            .client
            .post(format!("{}:{}/add", self.hostname, self.port).as_str())
            .json(&task_name)
            .send()
            .await;

        match result {
            Ok(result) => {
                let status_code = result.status();
                let result = result.json::<Task>().await;
                match result {
                    Ok(result) => Ok((status_code, result)),
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(err),
        }
    }

    /// Retrieve the index of a task by name from the server.
    ///
    /// Sends the task name as JSON to the server's `/index` endpoint and returns the HTTP status together with the parsed index on success.
    ///
    /// # Parameters
    ///
    /// - `task_name`: the name of the task to locate.
    ///
    /// # Returns
    ///
    /// `(StatusCode, usize)` where `usize` is the index of the task returned by the server, and `StatusCode` is the HTTP response status.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use yesser_todo_api::Client;
    /// use std::string::String;
    ///
    /// # async fn run_example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://127.0.0.1".into(), None);
    /// let (status, index) = client.get_index(&"example-task".into()).await?;
    /// println!("status: {}, index: {}", status, index);
    /// # Ok(()) }
    /// ```
    pub async fn get_index(&self, task_name: &str) -> Result<(StatusCode, usize), Error> {
        let result = self
            .client
            .get(format!("{}:{}/index", self.hostname, self.port).as_str())
            .json(&task_name)
            .send()
            .await;
        match result {
            Ok(result) => {
                let status_code = result.status();
                let result = result.json::<usize>().await;
                match result {
                    Ok(result) => Ok((status_code, result)),
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(err),
        }
    }

    /// Remove a task identified by name from the remote server.
    ///
    /// This resolves the task's index on the server and requests deletion of that index.
    /// If the index lookup returns a non-OK HTTP status, that status is returned unchanged.
    /// Network or request errors are propagated as `Err`.
    ///
    /// # Returns
    ///
    /// `Ok(StatusCode)` containing the server's status for the delete request, or the non-OK
    /// status returned by the index lookup; `Err(Error)` on request/transport failures.
    ///
    /// # Examples
    ///
    /// ```
    /// # use yesser_todo_api::Client;
    /// # use tokio;
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::new("http://127.0.0.1".to_string(), None);
    ///     let status = client.remove(&"example-task".to_string()).await;
    ///     // handle result...
    ///     let _ = status;
    /// }
    /// ```
    pub async fn remove(&self, task_name: &str) -> Result<StatusCode, Error> {
        let index_result = self.get_index(task_name).await;
        let index: usize;
        match index_result {
            Ok((status_code, result)) => {
                if status_code != StatusCode::OK {
                    return Ok(status_code);
                }
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
            Ok(result) => Ok(result.status()),
            Err(err) => Err(err),
        }
    }

    /// Marks the task with the given name as done and returns the HTTP status and the updated task.
    ///
    /// If retrieving the task index returns a non-OK status, the function returns that status along with a `Task` whose `name` is `"Something went wrong"` and `done` is `false`.
    ///
    /// # Returns
    ///
    /// `(StatusCode, Task)` containing the response status and the task as returned by the server.
    ///
    /// # Examples
    ///
    /// ```
    /// # use yesser_todo_db::Task;
    /// # use reqwest::StatusCode;
    /// # async fn _example() {
    /// let client = crate::Client::new("http://127.0.0.1".to_string(), None);
    /// let res = client.done(&"test".to_string()).await;
    /// match res {
    ///     Ok((status, task)) => {
    ///         assert!(status == StatusCode::OK || status.is_client_error() || status.is_server_error());
    ///         // `task` is the updated task from the server
    ///         let _ = task.name;
    ///     }
    ///     Err(e) => panic!("request failed: {:?}", e),
    /// }
    /// # }
    /// ```
    pub async fn done(&self, task_name: &str) -> Result<(StatusCode, Task), Error> {
        let index_result = self.get_index(task_name).await;
        let index: usize;
        match index_result {
            Ok((status_code, result)) => {
                if status_code != StatusCode::OK {
                    // TODO: Replace with error
                    return Ok((
                        status_code,
                        Task {
                            name: "Something went wrong".to_string(),
                            done: false,
                        },
                    ));
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
                match result.json::<Task>().await {
                    Ok(result) => Ok((status_code, result)),
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(err),
        }
    }

    /// Mark the task identified by `task_name` as not done and return the updated task with the response status.
    ///
    /// Attempts to resolve the task's index by name; if index resolution returns a non-OK status, returns that status
    /// together with a placeholder `Task` having name `"Something went wrong"` and `done: false`.
    ///
    /// # Returns
    ///
    /// `(StatusCode, Task)` with the HTTP response status and the updated task on success; if index lookup returns a non-OK status,
    /// returns that status paired with a placeholder `Task`.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_api::Client;
    /// use std::string::String;
    /// use reqwest::StatusCode;
    ///
    /// let client = Client::new("http://127.0.0.1", None);
    /// let rt = tokio::runtime::Runtime::new().unwrap();
    /// let res = rt.block_on(async { client.undone(&"example".to_string()).await }).unwrap();
    /// assert!(matches!(res.0, StatusCode::OK) || res.0.is_client_error() || res.0.is_server_error());
    /// ```
    pub async fn undone(&self, task_name: &str) -> Result<(StatusCode, Task), Error> {
        let index_result = self.get_index(task_name).await;
        let index: usize;
        match index_result {
            Ok((status_code, result)) => {
                if status_code != StatusCode::OK {
                    return Ok((
                        status_code,
                        Task {
                            name: "Something went wrong".to_string(),
                            done: false,
                        },
                    ));
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
                match result.json::<Task>().await {
                    Ok(result) => Ok((status_code, result)),
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(err),
        }
    }

    /// Clears all tasks on the remote to-do service.
    ///
    /// Sends a DELETE request to the configured `/clear` endpoint and returns the HTTP status code.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example() {
    /// let client = Client::new("http://127.0.0.1".to_string(), None);
    /// let status = client.clear().await.unwrap();
    /// assert_eq!(status, reqwest::StatusCode::OK);
    /// # }
    /// ```
    pub async fn clear(&self) -> Result<StatusCode, Error> {
        let result = self.client.delete(format!("{}:{}/clear", self.hostname, self.port).as_str()).send().await;
        match result {
            Ok(result) => Ok(result.status()),
            Err(err) => Err(err),
        }
    }

    /// Deletes all tasks marked as done on the remote to-do service.
    ///
    /// On success returns the HTTP response status code from the server; on failure returns the underlying `reqwest::Error`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use api::Client;
    ///
    /// let client = Client::new("http://127.0.0.1".into(), None);
    /// let status = tokio::runtime::Runtime::new()
    ///     .unwrap()
    ///     .block_on(client.clear_done())
    ///     .unwrap();
    /// assert!(status.is_success());
    /// ```
    pub async fn clear_done(&self) -> Result<StatusCode, Error> {
        let result = self.client.delete(format!("{}:{}/cleardone", self.hostname, self.port).as_str()).send().await;
        match result {
            Ok(result) => Ok(result.status()),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get() {
        let client = Client::new("http://127.0.0.1", None);
        let result = client.get().await;
        println!("{:?}", result);
        assert!(result.is_ok() && result.unwrap().0 == StatusCode::OK);
    }

    #[tokio::test]
    async fn add_get_index_done_undone_remove() {
        let client = Client::new("http://127.0.0.1", None);
        // add
        let result = client.add(&"test".to_string()).await;
        println!("{:?}", result);
        assert!(result.is_ok() && result.unwrap().0 == StatusCode::OK);
        // get_index
        let result = client.get_index(&"test".to_string()).await;
        println!("{:?}", result);
        assert!(result.is_ok() && result.unwrap().0 == StatusCode::OK);
        // done
        let result = client.done(&"test".to_string()).await;
        println!("{:?}", result);
        assert!(result.is_ok() && result.unwrap().0 == StatusCode::OK);
        // undone
        let result = client.undone(&"test".to_string()).await;
        println!("{:?}", result);
        assert!(result.is_ok() && result.unwrap().0 == StatusCode::OK);
        // remove
        let result = client.remove(&"test".to_string()).await;
        println!("{:?}", result);
        assert!(result.is_ok() && result.unwrap() == StatusCode::OK);
    }

    #[tokio::test]
    async fn clear() {
        let client = Client::new("http://127.0.0.1", None);
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
        assert!(unwrapped.0 == StatusCode::OK && unwrapped.1.len() == 0);
    }

    #[tokio::test]
    async fn clear_done() {
        let client = Client::new("http://127.0.0.1", None);
        let _ = client.add(&"test1".to_string()).await;
        let _ = client.add(&"test2".to_string()).await;
        let _ = client.add(&"test3".to_string()).await;
        let _ = client.done(&"test1".to_string()).await;
        let _ = client.done(&"test3".to_string()).await;
        let result = client.clear_done().await;
        println!("{:?}", result);
        assert!(result.is_ok());
        let result = client.get().await;
        println!("{:?}", result);
        assert!(result.is_ok());
        let unwrapped = result.unwrap();
        assert!(unwrapped.0 == StatusCode::OK && unwrapped.1.len() == 1 && unwrapped.1[0].name == "test2");
    }
}
