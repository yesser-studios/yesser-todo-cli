use std::fmt::{self, Display, Formatter};

use thiserror::Error;
use yesser_todo_db::db_error::DatabaseError;

#[derive(Debug, Error)]
pub(crate) enum CommandError {
    NoTasksSpecified,
    TaskExists { name: String },
    TaskNotFound { name: String },
    DuplicateInput { name: String },
    DataError { what: String, err: DatabaseError },
    HTTPError { name: String, status_code: u16 },
    ConnectionError { name: String },
    UnlinkedError,
}

impl CommandError {
    pub(crate) fn handle(&self) {
        eprintln!("{self}")
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::NoTasksSpecified => write!(f, "No tasks specified!"),
            CommandError::TaskExists { name } => write!(f, "Task {name} already exists!"),
            CommandError::TaskNotFound { name } => write!(f, "Task {name} not found!"),
            CommandError::DuplicateInput { name } => write!(f, "Task {name} was specified multiple times!"),
            CommandError::DataError { what, err } => write!(f, "Unable to save {what}: {err}!"),
            CommandError::HTTPError { name, status_code } => {
                write!(
                    f,
                    "HTTP error code {status_code}{}!",
                    if name.is_empty() { "".into() } else { format!(" for task {name}") }
                )
            }
            CommandError::ConnectionError { name } => {
                write!(
                    f,
                    "Failed to connect to the server{}!",
                    if name.is_empty() { "".into() } else { format!(" for task {name}") }
                )
            }
            CommandError::UnlinkedError => write!(f, "You're already unlinked!"),
        }
    }
}
