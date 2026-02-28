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










        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_error_display_400() {
        let err = ApiError::HTTPError(StatusCode::BAD_REQUEST);
        assert_eq!(format!("{}", err), "Server returned HTTP error code 400 Bad Request");
    }

    #[test]
    fn test_http_error_display_404() {
        let err = ApiError::HTTPError(StatusCode::NOT_FOUND);
        assert_eq!(format!("{}", err), "Server returned HTTP error code 404 Not Found");
    }

    #[test]
    fn test_http_error_display_500() {
        let err = ApiError::HTTPError(StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(format!("{}", err), "Server returned HTTP error code 500 Internal Server Error");
    }

    #[tokio::test]
    async fn test_request_error_display() {
        let reqwest_err = reqwest::get("http://invalid-domain-that-does-not-exist-12345.com").await.unwrap_err();
        let err = ApiError::RequestError(reqwest_err);
        assert_eq!(format!("{}", err), "Failed to connect to server");
    }

    #[test]
    fn test_http_error_debug() {
        let err = ApiError::HTTPError(StatusCode::BAD_REQUEST);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("HTTPError"));
    }

    #[tokio::test]
    async fn test_request_error_debug() {
        let reqwest_err = reqwest::Client::new().get("http://invalid").send().await.unwrap_err();
        let err = ApiError::RequestError(reqwest_err);
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("RequestError"));
    }

    #[test]
    fn test_http_error_multiple_status_codes() {
        let status_codes = vec![
            StatusCode::OK,
            StatusCode::CREATED,
            StatusCode::UNAUTHORIZED,
            StatusCode::FORBIDDEN,
            StatusCode::SERVICE_UNAVAILABLE,
        ];
        for status in status_codes {
            let err = ApiError::HTTPError(status);
            let msg = format!("{}", err);
            assert!(msg.starts_with("Server returned HTTP error code"));
        }
    }

    #[test]
    fn test_error_trait_implementation() {
        let err = ApiError::HTTPError(StatusCode::BAD_REQUEST);
        let _: &dyn std::error::Error = &err;
    }
}