use std::fmt::Display;

use reqwest::StatusCode;
use thiserror::Error;

use crate::server_error::ServerError;

#[derive(Debug, Error)]
pub enum ApiError {
    ServerError(ServerError),
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
    /// use yesser_todo_errors::api_error::ApiError;
    /// // `ApiError` is defined in the current crate module where this formatter lives.
    /// let err = ApiError::HTTPError(StatusCode::BAD_REQUEST);
    /// assert_eq!(format!("{}", err), format!("Server returned HTTP error code {}",
    /// StatusCode::BAD_REQUEST));
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServerError(err) => err.fmt(f),
            Self::HTTPError(status_code) => write!(f, "Server returned HTTP error code {status_code}"),
            Self::RequestError(_) => write!(f, "Failed to connect to server"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_error_display_bad_request() {
        let err = ApiError::HTTPError(StatusCode::BAD_REQUEST);
        assert_eq!(format!("{}", err), "Server returned HTTP error code 400 Bad Request");
    }

    #[test]
    fn test_http_error_display_not_found() {
        let err = ApiError::HTTPError(StatusCode::NOT_FOUND);
        assert_eq!(format!("{}", err), "Server returned HTTP error code 404 Not Found");
    }

    #[test]
    fn test_http_error_display_internal_server_error() {
        let err = ApiError::HTTPError(StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(format!("{}", err), "Server returned HTTP error code 500 Internal Server Error");
    }

    #[test]
    fn test_http_error_display_unauthorized() {
        let err = ApiError::HTTPError(StatusCode::UNAUTHORIZED);
        assert_eq!(format!("{}", err), "Server returned HTTP error code 401 Unauthorized");
    }

    #[test]
    fn test_api_error_is_error_trait() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<ApiError>();
    }

    #[test]
    fn test_api_error_debug() {
        let err = ApiError::HTTPError(StatusCode::BAD_REQUEST);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("HTTPError"));
        assert!(debug_str.contains("400"));
    }

    #[test]
    fn test_http_error_various_status_codes() {
        let test_cases = vec![
            (StatusCode::OK, "Server returned HTTP error code 200 OK"),
            (StatusCode::CREATED, "Server returned HTTP error code 201 Created"),
            (StatusCode::FORBIDDEN, "Server returned HTTP error code 403 Forbidden"),
            (StatusCode::SERVICE_UNAVAILABLE, "Server returned HTTP error code 503 Service Unavailable"),
        ];

        for (status, expected) in test_cases {
            let err = ApiError::HTTPError(status);
            assert_eq!(format!("{}", err), expected);
        }
    }

    #[test]
    fn test_server_error_display_not_found() {
        use crate::server_error::TaskSelector;
        let err = ApiError::ServerError(ServerError::NotFound(TaskSelector::Name("test task".into())));
        assert_eq!(format!("{}", err), "Task test task not found!");
    }

    #[test]
    fn test_server_error_display_conflict() {
        use crate::server_error::TaskSelector;
        let err = ApiError::ServerError(ServerError::Conflict(TaskSelector::Index(1)));
        assert_eq!(format!("{}", err), "Task of index 1 already exists!");
    }

    #[test]
    fn test_server_error_display_io_error() {
        let err = ApiError::ServerError(ServerError::IOError("database connection failed".into()));
        assert_eq!(format!("{}", err), "IO error: database connection failed");
    }
}
