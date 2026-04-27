pub mod helpers;

use yesser_todo_db::Task;

pub use yesser_todo_errors::api_error::ApiError;
use yesser_todo_errors::server_error::ServerError;

use crate::helpers::is_success;

pub const DEFAULT_PORT: &str = "6982";

pub struct Client {
    pub hostname: String,
    pub port: String,
    client: ureq::Agent,
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
                client: ureq::Agent::new_with_defaults(),
            },
            Some(port) => Client {
                hostname,
                port,
                client: ureq::Agent::new_with_defaults(),
            },
        }
    }

    /// Retrieves all tasks from the configured server.
    ///
    /// # Returns
    ///
    /// A tuple `(u16, Vec<Task>)` where `u16` is the HTTP response status and `Vec<Task>` is the list of tasks.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use yesser_todo_api::Client;
    /// use yesser_todo_db::Task;
    ///
    /// # fn example() -> Option<Vec<Task>> {
    /// let client = Client::new("http://127.0.0.1".into(), None);
    /// let (status, tasks) = client.get().ok()?;
    /// // `tasks` is a Vec<yesser_todo_db::Task>
    /// # Some(tasks)
    /// # }
    /// ```
    pub fn get(&self) -> Result<(u16, Vec<Task>), ApiError> {
        let mut result = self.client.get(format!("{}:{}/tasks", self.hostname, self.port).as_str()).call()?;

        let status_code = result.status();
        if status_code.is_success() {
            let result = result.body_mut().read_json::<Vec<Task>>();
            match result {
                Ok(result) => Ok((status_code.as_u16(), result)),
                Err(err) => Err(ApiError::RequestError(err)),
            }
        } else {
            match result.body_mut().read_json::<ServerError>() {
                Ok(err) => Err(ApiError::ServerError(err)),
                Err(_) => Err(ApiError::HTTPError(status_code.as_u16())),
            }
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
    /// use yesser_todo_api::{ Client, helpers::is_success };
    ///
    /// fn example_add() {
    ///     let client = Client::new("http://127.0.0.1".to_string(), None);
    ///     let (status, task) = client.add("example task").unwrap();
    ///     assert!(is_success(status));
    ///     assert_eq!(task.name, "example task");
    /// }
    /// ```
    pub fn add(&self, task_name: &str) -> Result<(u16, Task), ApiError> {
        let mut result = self.client.post(format!("{}:{}/add", self.hostname, self.port).as_str()).send_json(task_name)?;

        let status_code = result.status();
        if status_code.is_success() {
            let result = result.body_mut().read_json::<Task>();
            match result {
                Ok(result) => Ok((status_code.as_u16(), result)),
                Err(err) => Err(ApiError::RequestError(err)),
            }
        } else {
            match result.body_mut().read_json::<ServerError>() {
                Ok(err) => Err(ApiError::ServerError(err)),
                Err(_) => Err(ApiError::HTTPError(status_code.as_u16())),
            }
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
    /// # fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://127.0.0.1".into(), None);
    /// let (status, index) = client.get_index("example-task")?;
    /// println!("status: {}, index: {}", status, index);
    /// # Ok(()) }
    /// ```
    pub fn get_index(&self, task_name: &str) -> Result<(u16, usize), ApiError> {
        let mut result = self
            .client
            .get(format!("{}:{}/index", self.hostname, self.port).as_str())
            .query("name", task_name)
            .call()?;
        let status_code = result.status();
        if status_code.is_success() {
            let result = result.body_mut().read_json::<usize>();
            match result {
                Ok(result) => Ok((status_code.as_u16(), result)),
                Err(err) => Err(ApiError::RequestError(err)),
            }
        } else {
            match result.body_mut().read_json::<ServerError>() {
                Ok(err) => Err(ApiError::ServerError(err)),
                Err(_) => Err(ApiError::HTTPError(status_code.as_u16())),
            }
        }
    }

    /// Delete the task with the given name from the remote server.
    ///
    /// Resolves the task's index on the server and sends a DELETE request for that index.
    ///
    /// # Returns
    ///
    /// `Ok(u16)` if the server responded with a success status for the delete request, `Err(ApiError)` on transport or HTTP error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use yesser_todo_api::Client;
    /// fn main() {
    ///     let client = Client::new("http://127.0.0.1".to_string(), None);
    ///     let _status = client.remove("example-task");
    /// }
    /// ```
    pub fn remove(&self, task_name: &str) -> Result<u16, ApiError> {
        let index_result = self.get_index(task_name);
        let index = match index_result {
            Ok((_, result)) => result,
            Err(err) => return Err(err),
        };

        let mut result = self
            .client
            .delete(format!("{}:{}/remove", self.hostname, self.port).as_str())
            .query("index", index.to_string())
            .call()?;
        if result.status().is_success() {
            Ok(result.status().as_u16())
        } else {
            let status_code = result.status();
            match result.body_mut().read_json::<ServerError>() {
                Ok(err) => Err(ApiError::ServerError(err)),
                Err(_) => Err(ApiError::HTTPError(status_code.as_u16())),
            }
        }
    }

    /// Mark the task with the given name as done and return the server status and the updated task.
    ///
    /// On success returns a tuple containing the response `u16` status code and the `Task` as returned by the server.
    /// On failure returns an `Err(ApiError)`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use yesser_todo_api::Client;
    /// # use yesser_todo_db::Task;
    /// # fn _example() {
    /// let client = Client::new("http://127.0.0.1".to_string(), None);
    /// let res = client.done("test");
    /// match res {
    ///     Ok((status, task)) => {
    ///         let _ = task.name;
    ///     }
    ///     Err(e) => panic!("request failed: {:?}", e),
    /// }
    /// # }
    /// ```
    pub fn done(&self, task_name: &str) -> Result<(u16, Task), ApiError> {
        let index = match self.get_index(task_name) {
            Ok((status_code, result)) => {
                if !is_success(status_code) {
                    return Err(ApiError::HTTPError(status_code));
                }
                result
            }
            Err(err) => return Err(err),
        };
        let mut result = self.client.post(format!("{}:{}/done", self.hostname, self.port).as_str()).send_json(index)?;
        let status_code = result.status();
        if status_code.is_success() {
            let result = result.body_mut().read_json::<Task>();
            match result {
                Ok(result) => Ok((status_code.as_u16(), result)),
                Err(err) => Err(ApiError::RequestError(err)),
            }
        } else {
            let status_code = result.status();
            match result.body_mut().read_json::<ServerError>() {
                Ok(err) => Err(ApiError::ServerError(err)),
                Err(_) => Err(ApiError::HTTPError(status_code.as_u16())),
            }
        }
    }

    /// Mark the task identified by `task_name` as not done and return the updated task.
    ///
    /// # Returns
    ///
    /// `Ok((u16, Task))` with the HTTP response status and the updated task on success, `Err(ApiError)` on failure.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use yesser_todo_api::Client;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = Client::new("http://127.0.0.1".to_string(), None);
    /// let res = client.undone("example")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn undone(&self, task_name: &str) -> Result<(u16, Task), ApiError> {
        let index_result = self.get_index(task_name);
        let index = match index_result {
            Ok((status_code, result)) => {
                if !is_success(status_code) {
                    return Err(ApiError::HTTPError(status_code));
                }
                result
            }
            Err(err) => return Err(err),
        };

        let mut result = self.client.post(format!("{}:{}/undone", self.hostname, self.port).as_str()).send_json(index)?;
        let status_code = result.status();
        if status_code.is_success() {
            let result = result.body_mut().read_json::<Task>();
            match result {
                Ok(result) => Ok((status_code.as_u16(), result)),
                Err(err) => Err(ApiError::RequestError(err)),
            }
        } else {
            let status_code = result.status();
            match result.body_mut().read_json::<ServerError>() {
                Ok(err) => Err(ApiError::ServerError(err)),
                Err(_) => Err(ApiError::HTTPError(status_code.as_u16())),
            }
        }
    }

    /// Clears all tasks on the remote to-do service.
    ///
    /// Sends a DELETE request to the configured `/clear` endpoint and returns the HTTP status code.
    ///
    /// # Returns
    /// - status code of the request;
    /// - on failure returns `Err(ApiError)`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use yesser_todo_api::Client;
    /// # fn example() {
    /// let client = Client::new("http://127.0.0.1".to_string(), None);
    /// let status = client.clear().unwrap();
    /// assert_eq!(status, 200);
    /// # }
    /// ```
    pub fn clear(&self) -> Result<u16, ApiError> {
        let result = self.client.delete(format!("{}:{}/clear", self.hostname, self.port).as_str()).call()?;
        if result.status().is_success() {
            Ok(result.status().as_u16())
        } else {
            Err(ApiError::HTTPError(result.status().as_u16()))
        }
    }

    /// Clears all tasks marked as done on the remote to-do service.
    ///
    /// Sends a DELETE request to "{hostname}:{port}/cleardone".
    ///
    /// # Returns
    ///
    /// `Ok(u16)` with the HTTP response status when the request succeeds, `Err(ApiError)` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use yesser_todo_api::Client;
    /// use yesser_todo_api::helpers::is_success;
    ///
    /// let client = Client::new("http://127.0.0.1".into(), None);
    /// let status = client.clear_done().unwrap();
    /// # assert!(is_success(status));
    /// ```
    pub fn clear_done(&self) -> Result<u16, ApiError> {
        let result = self.client.delete(format!("{}:{}/cleardone", self.hostname, self.port).as_str()).call()?;
        if result.status().is_success() {
            Ok(result.status().as_u16())
        } else {
            Err(ApiError::HTTPError(result.status().as_u16()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get() {
        let client = Client::new("http://127.0.0.1".to_string(), None);
        let result = client.get();
        println!("{:?}", result);
        assert!(result.is_ok() && is_success(result.unwrap().0));
    }

    #[test]
    fn add_get_index_done_undone_remove() {
        let client = Client::new("http://127.0.0.1".to_string(), None);
        // add
        let result = client.add("test");
        println!("{:?}", result);
        assert!(result.is_ok() && is_success(result.unwrap().0));
        // get_index
        let result = client.get_index("test");
        println!("{:?}", result);
        assert!(result.is_ok() && is_success(result.unwrap().0));
        // done
        let result = client.done("test");
        println!("{:?}", result);
        assert!(result.is_ok() && is_success(result.unwrap().0));
        // undone
        let result = client.undone("test");
        println!("{:?}", result);
        assert!(result.is_ok() && is_success(result.unwrap().0));
        // remove
        let result = client.remove("test");
        println!("{:?}", result);
        assert!(result.is_ok() && is_success(result.unwrap()));

        // cleanup
        let result = client.clear();
        println!("{:?}", result);
        assert!(result.is_ok() && is_success(result.unwrap()));
    }

    #[test]
    fn clear() {
        let client = Client::new("http://127.0.0.1".to_string(), None);
        let _ = client.add("test");
        let _ = client.add("test");
        let _ = client.add("test");
        let result = client.clear();
        println!("{:?}", result);
        assert!(result.is_ok());
        let result = client.get();
        println!("{:?}", result);
        assert!(result.is_ok());
        let unwrapped = result.unwrap();
        assert!(is_success(unwrapped.0) && unwrapped.1.is_empty());
    }

    #[test]
    fn clear_done() {
        let client = Client::new("http://127.0.0.1".to_string(), None);
        let _ = client.add("test1");
        let _ = client.add("test2");
        let _ = client.add("test3");
        let _ = client.done("test1");
        let _ = client.done("test3");
        let result = client.clear_done();
        println!("{:?}", result);
        assert!(result.is_ok());
        let result = client.get();
        println!("{:?}", result);
        assert!(result.is_ok());
        let unwrapped = result.unwrap();
        assert!(is_success(unwrapped.0) && unwrapped.1.len() == 1 && unwrapped.1[0].name == "test2");

        // cleanup
        let result = client.clear();
        println!("{:?}", result);
        assert!(result.is_ok() && is_success(result.unwrap()));
    }
}
