use std::fmt::Display;

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::db_error::DatabaseError;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ServerError {
    NotFound(TaskSelector),
    Conflict(TaskSelector),
    IOError(String),
}

impl ServerError {
    pub fn to_status_code(&self) -> StatusCode {
        match self {
            ServerError::NotFound(_) => StatusCode::NOT_FOUND,
            ServerError::Conflict(_) => StatusCode::CONFLICT,
            ServerError::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<DatabaseError> for ServerError {
    fn from(value: DatabaseError) -> Self {
        Self::IOError(format!("{value}"))
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::NotFound(selector) => write!(f, "{selector} not found!"),
            ServerError::Conflict(selector) => write!(f, "{selector} already exists!"),
            ServerError::IOError(error) => write!(f, "IO error: {error}"),
        }
    }
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        let status = self.to_status_code();
        (status, Json(self)).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TaskSelector {
    Index(usize),
    Name(String),
}

impl Display for TaskSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskSelector::Index(index) => write!(f, "Task of index {index}"),
            TaskSelector::Name(name) => write!(f, "Task {name}"),
        }
    }
}

impl From<TaskSelector> for Option<String> {
    fn from(val: TaskSelector) -> Self {
        match val {
            TaskSelector::Index(_) => None,
            TaskSelector::Name(name) => Some(name),
        }
    }
}
impl From<TaskSelector> for Option<usize> {
    fn from(val: TaskSelector) -> Self {
        match val {
            TaskSelector::Index(index) => Some(index),
            TaskSelector::Name(_) => None,
        }
    }
}

impl From<usize> for TaskSelector {
    fn from(value: usize) -> Self {
        Self::Index(value)
    }
}
impl From<String> for TaskSelector {
    fn from(value: String) -> Self {
        Self::Name(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_status_code_not_found() {
        let err = ServerError::NotFound(TaskSelector::Name("test".into()));
        assert_eq!(err.to_status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_to_status_code_conflict() {
        let err = ServerError::Conflict(TaskSelector::Index(1));
        assert_eq!(err.to_status_code(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_to_status_code_io_error() {
        let err = ServerError::IOError("test error".into());
        assert_eq!(err.to_status_code(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_display_not_found() {
        let err = ServerError::NotFound(TaskSelector::Name("my task".into()));
        assert_eq!(format!("{}", err), "Task my task not found!");
    }

    #[test]
    fn test_display_conflict_index() {
        let err = ServerError::Conflict(TaskSelector::Index(5));
        assert_eq!(format!("{}", err), "Task of index 5 already exists!");
    }

    #[test]
    fn test_display_io_error() {
        let err = ServerError::IOError("disk full".into());
        assert_eq!(format!("{}", err), "IO error: disk full");
    }

    #[test]
    fn test_task_selector_display_index() {
        let selector = TaskSelector::Index(42);
        assert_eq!(format!("{}", selector), "Task of index 42");
    }

    #[test]
    fn test_task_selector_display_name() {
        let selector = TaskSelector::Name("buy milk".into());
        assert_eq!(format!("{}", selector), "Task buy milk");
    }
}
