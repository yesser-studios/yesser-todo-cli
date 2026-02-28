use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    JsonError(serde_json::Error),
    IOError(std::io::Error),
    UserDirsError,
}

impl From<serde_json::Error> for DatabaseError {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonError(value)
    }
}

impl From<std::io::Error> for DatabaseError {
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err)
    }
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::JsonError(error) => write!(f, "A JSON Deserialization error occured: {error}"),
            DatabaseError::IOError(error) => write!(f, "An error occured while writing to config file or directory: {error}"),
            DatabaseError::UserDirsError => write!(f, "Could not get user config directory location"),
        }
    }
}
