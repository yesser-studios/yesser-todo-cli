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
        let err = CommandError::TaskExists { name: "test task".to_string() };
        assert_eq!(format!("{}", err), "Task test task already exists!");
    }

    #[test]
    fn test_task_not_found_display() {
        let err = CommandError::TaskNotFound { name: "missing task".to_string() };
        assert_eq!(format!("{}", err), "Task missing task not found!");
    }

    #[test]
    fn test_duplicate_input_display() {
        let err = CommandError::DuplicateInput { name: "duplicate".to_string() };
        assert_eq!(format!("{}", err), "Task duplicate was specified multiple times!");
    }

    #[test]
    fn test_data_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let db_err = DatabaseError::IOError(io_err);
        let err = CommandError::DataError {
            what: "configuration".to_string(),
            err: db_err,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Unable to save configuration"));
    }

    #[test]
    fn test_http_error_with_name() {
        let err = CommandError::HTTPError {
            name: "test task".to_string(),
            status_code: 404,
        };
        assert_eq!(format!("{}", err), "HTTP error code: 404 for task test task");
    }

    #[test]
    fn test_http_error_without_name() {
        let err = CommandError::HTTPError {
            name: "".to_string(),
            status_code: 500,
        };
        assert_eq!(format!("{}", err), "HTTP error code: 500!");
    }

    #[test]
    fn test_connection_error_with_name() {
        let err = CommandError::ConnectionError { name: "my task".to_string() };
        assert_eq!(format!("{}", err), "Failed to connect to the server for task my task");
    }

    #[test]
    fn test_connection_error_without_name() {
        let err = CommandError::ConnectionError { name: "".to_string() };
        assert_eq!(format!("{}", err), "Failed to connect to the server!");
    }

    #[test]
    fn test_unlinked_error_display() {
        let err = CommandError::UnlinkedError;
        assert_eq!(format!("{}", err), "You're already unlinked!");
    }

    #[test]
    fn test_error_debug() {
        let err = CommandError::NoTasksSpecified;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("NoTasksSpecified"));
    }

    #[test]
    fn test_error_trait_implementation() {
        let err = CommandError::NoTasksSpecified;
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_http_error_various_codes() {
        let codes = vec![200, 201, 400, 401, 403, 404, 500, 502, 503];
        for code in codes {
            let err = CommandError::HTTPError {
                name: "task".to_string(),
                status_code: code,
            };
            let msg = format!("{}", err);
            assert!(msg.contains(&code.to_string()));
        }
    }
}