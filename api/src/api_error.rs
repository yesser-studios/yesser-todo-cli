use reqwest::{Error, StatusCode};

#[derive(Debug)]
pub enum ApiError {
    HTTPError(StatusCode),
    RequestError(Error),
}
