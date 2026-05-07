use std::fmt::Display;

use thiserror::Error;

use crate::server_error::ServerError;

#[derive(Debug, Error)]
pub enum ApiError {
    ServerError(ServerError),
    HTTPError(u16),
    #[cfg(feature = "client")]
    RequestError(ureq::Error),
}

#[cfg(feature = "client")]
impl From<ureq::Error> for ApiError {
    fn from(value: ureq::Error) -> Self {
        ApiError::RequestError(value)
    }
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
    /// use yesser_todo_errors::api_error::ApiError;
    /// // `ApiError` is defined in the current crate module where this formatter lives.
    /// let err = ApiError::HTTPError(400);
    /// assert_eq!(format!("{}", err), format!("Server returned HTTP error code {}", 400));
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServerError(err) => err.fmt(f),
            Self::HTTPError(status_code) => write!(f, "Server returned HTTP error code {status_code}"),
            #[cfg(feature = "client")]
            Self::RequestError(_) => write!(f, "Failed to connect to server"),
        }
    }
}

#[cfg(feature = "client")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_is_error_trait() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<ApiError>();
    }

    #[test]
    fn test_api_error_debug() {
        let err = ApiError::HTTPError(400);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("HTTPError"));
        assert!(debug_str.contains("400"));
    }

    #[test]
    fn test_http_error_various_status_codes() {
        let test_cases = vec![
            (200, "Server returned HTTP error code 200"),
            (201, "Server returned HTTP error code 201"),
            (403, "Server returned HTTP error code 403"),
            (503, "Server returned HTTP error code 503"),
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
