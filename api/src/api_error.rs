use std::fmt::Display;

use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    HTTPError(StatusCode),
    RequestError(reqwest::Error),
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::HTTPError(status_code) => write!(f, "Server returned HTTP error code {status_code}"),
            ApiError::RequestError(_) => write!(f, "Failed to connect to server"),
        }
    }
}
