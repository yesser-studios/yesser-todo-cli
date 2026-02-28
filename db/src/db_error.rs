use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("A JSON deserialization error occured: {0}")]
    JsonError(serde_json::Error),

    #[error("An error occurred while writing to config file or directory: {0}")]
    IOError(std::io::Error),

    #[error("Could not get user config directory location")]
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
