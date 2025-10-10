use reqwest::{Error, Response};
use yesser_todo_db::Task;

pub struct Client {
    pub hostname: String,
    pub port: String,
    client: reqwest::Client,
}

impl Client {
    pub fn new(hostname: String, port: String) -> Client {
        return Client{hostname, port, client: reqwest::Client::new(), };
    }

    async fn get(&self) -> Result<Vec<Task>, Error> {
        let result = self.client
            .get(format!("https://{}:{}/tasks", self.hostname, self.port).as_str())
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
}

#[cfg(test)]
mod tests {
    use super::*;
}
