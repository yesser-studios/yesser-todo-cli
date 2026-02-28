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
    /// Prints the formatted error message to standard error.
    ///
    /// This uses the type's `Display` implementation to produce a user-facing message.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::CommandError;
    ///
    /// let err = CommandError::NoTasksSpecified;
    /// err.handle(); // prints "No tasks specified!" to stderr
    /// ```
    pub(crate) fn handle(&self) {
        eprintln!("{self}")
    }
}

impl Display for CommandError {
    /// Produates a human-readable, user-facing message for each `CommandError` variant.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::CommandError;
    ///
    /// assert_eq!(
    ///     format!("{}", CommandError::TaskNotFound { name: "foo".into() }),
    ///     "Task foo not found!"
    /// );
    /// assert_eq!(
    ///     format!("{}", CommandError::NoTasksSpecified),
    ///     "No tasks specified!"
    /// );
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::NoTasksSpecified => write!(f, "No tasks specified!"),
            CommandError::TaskExists { name } => write!(f, "Task {name} already exists!"),
            CommandError::TaskNotFound { name } => write!(f, "Task {name} not found!"),
            CommandError::DuplicateInput { name } => write!(f, "Task {name} was specified multiple times!"),
            CommandError::DataError { what, err } => write!(f, "Unable to save {what}: {err}!"),
            CommandError::HTTPError { name, status_code } => {
                if name.is_empty() {
                    write!(f, "HTTP error code: {status_code}!")
                } else {
                    write!(f, "HTTP error code: {status_code} for task {name}")
                }
            }
            CommandError::ConnectionError { name } => {
                if name.is_empty() {
                    write!(f, "Failed to connect to the server!")
                } else {
                    write!(f, "Failed to connect to the server for task {name}")
                }
            }
            CommandError::UnlinkedError => write!(f, "You're already unlinked!"),
        }
    }
}
