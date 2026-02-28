use std::fmt::Display;

use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    HTTPError(StatusCode),
    RequestError(reqwest::Error),
}

impl Display for ApiError {
    /// Formats an `ApiError` as a human-readable message.
    ///
    /// - `ApiError::HTTPError(status)` is displayed as `"Server returned HTTP error code {status}"`.
    /// - `ApiError::RequestError(_)` is displayed as `"Failed to connect to server"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use reqwest::StatusCode;
    /// use yesser_todo_api::api_error::ApiError;
    /// // `ApiError` is defined in the current crate module where this formatter lives.
    /// let err = ApiError::HTTPError(StatusCode::BAD_REQUEST);
    /// assert_eq!(format!("{}", err), format!("Server returned HTTP error code {}",
    /// StatusCode::BAD_REQUEST));
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::HTTPError(status_code) => write!(f, "Server returned HTTP error code {status_code}"),
            ApiError::RequestError(_) => write!(f, "Failed to connect to server"),
        }
    }
}
