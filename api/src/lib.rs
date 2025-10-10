use reqwest::Error;
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

    async fn get(&self) -> Result<Vec<Task>, Error> {
        let result = self.client
            .get(format!("{}:{}/tasks", self.hostname, self.port).as_str())
            .send().await;

        match result {
            Ok(result) => {
                let result = result.json::<Vec<Task>>().await;
                match result {
                    Ok(result) => {Ok(result)},
                    Err(err) => {Err(err)}
                }
            }
            Err(err) => {Err(err)}
        }
    }

    async fn add(&self, task: Task) -> Result<Task, Error> {
        let result = self.client
            .post(format!("{}:{}/add", self.hostname, self.port).as_str())
            .json(&task)
            .send().await;

        match result {
            Ok(result) => {
                let result = result.json::<Task>().await;
                match result {
                    Ok(result) => {Ok(result)},
                    Err(err) => {Err(err)}
                }
            }
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
        assert!(result.is_ok())
    }
}
