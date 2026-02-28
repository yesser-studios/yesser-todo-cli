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
    /// Produces a human-readable, user-facing message for each `CommandError` variant.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_tasks_specified_display() {
        let err = CommandError::NoTasksSpecified;
        assert_eq!(format!("{}", err), "No tasks specified!");
    }

    #[test]
    fn test_task_exists_display() {
        let err = CommandError::TaskExists {
            name: "test task".to_string(),
        };
        assert_eq!(format!("{}", err), "Task test task already exists!");
    }

    #[test]
    fn test_task_not_found_display() {
        let err = CommandError::TaskNotFound {
            name: "missing".to_string(),
        };
        assert_eq!(format!("{}", err), "Task missing not found!");
    }

    #[test]
    fn test_duplicate_input_display() {
        let err = CommandError::DuplicateInput {
            name: "duplicate".to_string(),
        };
        assert_eq!(format!("{}", err), "Task duplicate was specified multiple times!");
    }

    #[test]
    fn test_data_error_display() {
        let db_err = DatabaseError::UserDirsError;
        let err = CommandError::DataError {
            what: "tasks".to_string(),
            err: db_err,
        };
        let display = format!("{}", err);
        assert!(display.contains("Unable to save tasks"));
        assert!(display.contains("Could not get user config directory location"));
    }

    #[test]
    fn test_http_error_with_name() {
        let err = CommandError::HTTPError {
            name: "my-task".to_string(),
            status_code: 404,
        };
        assert_eq!(format!("{}", err), "HTTP error code: 404 for task my-task");
    }

    #[test]
    fn test_http_error_without_name() {
        let err = CommandError::HTTPError {
            name: String::new(),
            status_code: 500,
        };
        assert_eq!(format!("{}", err), "HTTP error code: 500!");
    }

    #[test]
    fn test_connection_error_with_name() {
        let err = CommandError::ConnectionError {
            name: "test-task".to_string(),
        };
        assert_eq!(format!("{}", err), "Failed to connect to the server for task test-task");
    }

    #[test]
    fn test_connection_error_without_name() {
        let err = CommandError::ConnectionError { name: String::new() };
        assert_eq!(format!("{}", err), "Failed to connect to the server!");
    }

    #[test]
    fn test_unlinked_error_display() {
        let err = CommandError::UnlinkedError;
        assert_eq!(format!("{}", err), "You're already unlinked!");
    }

    #[test]
    fn test_command_error_is_error_trait() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<CommandError>();
    }

    #[test]
    fn test_command_error_debug() {
        let err = CommandError::NoTasksSpecified;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NoTasksSpecified"));
    }

    #[test]
    fn test_various_http_status_codes() {
        let test_cases = vec![
            (200, "HTTP error code: 200!"),
            (400, "HTTP error code: 400!"),
            (404, "HTTP error code: 404!"),
            (500, "HTTP error code: 500!"),
        ];

        for (code, expected) in test_cases {
            let err = CommandError::HTTPError {
                name: String::new(),
                status_code: code,
            };
            assert_eq!(format!("{}", err), expected);
        }
    }

    #[test]
    fn test_task_names_with_special_characters() {
        let err = CommandError::TaskExists {
            name: "task with spaces & symbols!".to_string(),
        };
        assert_eq!(format!("{}", err), "Task task with spaces & symbols! already exists!");
    }

    #[test]
    fn test_data_error_with_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let db_err = DatabaseError::IOError(io_err);
        let err = CommandError::DataError {
            what: "config".to_string(),
            err: db_err,
        };
        let display = format!("{}", err);
        assert!(display.contains("Unable to save config"));
        assert!(display.contains("access denied"));
    }
}