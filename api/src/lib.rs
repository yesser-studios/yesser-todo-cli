use reqwest::{Error, StatusCode};
use std::string::ToString;
use yesser_todo_db::Task;

pub struct Client {
    pub hostname: String,
    pub port: String,
    client: reqwest::Client,
}

impl Client {
    pub fn new(hostname: String, port: Option<String>) -> Client {
        match port {
            None => {Client{hostname, port: "6982".to_string(), client: reqwest::Client::new(), }}
            Some(port) => {Client{hostname, port, client: reqwest::Client::new(), }}
        }
    }

    pub async fn get(&self) -> Result<(StatusCode, Vec<Task>), Error> {
        let result = self.client
            .get(format!("{}:{}/tasks", self.hostname, self.port).as_str())
            .send().await;

        match result {
            Ok(result) => {
                let status_code = result.status();
                let result = result.json::<Vec<Task>>().await;
                match result {
                    Ok(result) => {Ok((status_code, result))},
                    Err(err) => {Err(err)}
                }
            }
            Err(err) => {Err(err)}
        }
    }

    pub async fn add(&self, task_name: &String) -> Result<(StatusCode, Task), Error> {
        let result = self.client
            .post(format!("{}:{}/add", self.hostname, self.port).as_str())
            .json(&task_name)
            .send().await;

        match result {
            Ok(result) => {
                let status_code = result.status();
                let result = result.json::<Task>().await;
                match result {
                    Ok(result) => {Ok((status_code, result))},
                    Err(err) => {Err(err)}
                }
            }
            Err(err) => {Err(err)}
        }
    }

    pub async fn get_index(&self, task_name: &String) -> Result<(StatusCode, usize), Error> {
        let result = self.client
            .get(format!("{}:{}/index", self.hostname, self.port).as_str())
            .json(&task_name)
            .send().await;
        match result {
            Ok(result) => {
                let status_code = result.status();
                let result = result.json::<usize>().await;
                match result {
                    Ok(result) => Ok((status_code, result)),
                    Err(err) => {Err(err)}
                }
            }
            Err(err) => {Err(err)}
        }
    }

    pub async fn remove(&self, task_name: &String) -> Result<StatusCode, Error> {
        let index_result = self.get_index(task_name).await;
        let index: usize;
        match index_result {
            Ok((status_code, result)) => {
                if status_code != StatusCode::OK {
                    return Ok(status_code);
                }
                index = result;
            }
            Err(err) => {return Err(err)}
        }
        let result = self.client
            .delete(format!("{}:{}/remove", self.hostname, self.port).as_str())
            .json(&index)
            .send().await;
        match result {
            Ok(result) => {
                Ok(result.status())
            }
            Err(err) => {Err(err)}
        }
    }

    pub async fn done(&self, task_name: &String) -> Result<(StatusCode, Task), Error> {
        let index_result = self.get_index(task_name).await;
        let index: usize;
        match index_result {
            Ok((status_code, result)) => {
                if status_code != StatusCode::OK {
                    return Ok((status_code, Task{name: "Something went wrong".to_string(), done: false}));
                }
                index = result;
            }
            Err(err) => {return Err(err)}
        }
        let result = self.client
            .post(format!("{}:{}/done", self.hostname, self.port).as_str())
            .json(&index)
            .send().await;
        match result {
            Ok(result) => {
                let status_code = result.status();
                match result.json::<Task>().await {
                    Ok(result) => Ok((status_code, result)),
                    Err(err) => {Err(err)}
                }
            }
            Err(err) => {Err(err)}
        }
    }

    pub async fn undone(&self, task_name: &String) -> Result<(StatusCode, Task), Error> {
        let index_result = self.get_index(task_name).await;
        let index: usize;
        match index_result {
            Ok((status_code, result)) => {
                if status_code != StatusCode::OK {
                    return Ok((status_code, Task{name: "Something went wrong".to_string(), done: false}));
                }
                index = result;
            }
            Err(err) => {return Err(err)}
        }
        let result = self.client
            .post(format!("{}:{}/undone", self.hostname, self.port).as_str())
            .json(&index)
            .send().await;
        match result {
            Ok(result) => {
                let status_code = result.status();
                match result.json::<Task>().await {
                    Ok(result) => Ok((status_code, result)),
                    Err(err) => {Err(err)}
                }
            }
            Err(err) => {Err(err)}
        }
    }

    pub async fn clear(&self) -> Result<StatusCode, Error> {
        let result = self.client
            .delete(format!("{}:{}/clear", self.hostname, self.port).as_str())
            .send().await;
        match result {
            Ok(result) => Ok(result.status()),
            Err(err) => {Err(err)}
        }
    }

    pub async fn clear_done(&self) -> Result<StatusCode, Error> {
        let result = self.client
            .delete(format!("{}:{}/cleardone", self.hostname, self.port).as_str())
            .send().await;
        match result {
            Ok(result) => Ok(result.status()),
            Err(err) => {Err(err)}
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
        assert!(result.is_ok() && result.unwrap().0 == StatusCode::OK);
    }

    #[tokio::test]
    async fn add_get_index_done_undone_remove() {
        let client = Client::new("http://127.0.0.1".to_string(), None);
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
        let client = Client::new("http://127.0.0.1".to_string(), None);
        let _ = client.add(&"test".to_string()).await;
        let _ = client.add(&"test".to_string()).await;
        let _ = client.add(&"test".to_string()).await;
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
        let client = Client::new("http://127.0.0.1".to_string(), None);
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
        assert!(unwrapped.0 == StatusCode::OK
            && unwrapped.1.len() == 1
            && unwrapped.1[0].name == "test2");
    }
}
