use std::fmt::Display;

use axum::{Json, http::StatusCode, response::IntoResponse};
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
        let status = match &self {
            ServerError::NotFound(_) => StatusCode::NOT_FOUND,
            ServerError::Conflict(_) => StatusCode::CONFLICT,
            ServerError::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({"error": &self}))).into_response()
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
