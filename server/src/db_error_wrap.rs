use thiserror::Error;
use yesser_todo_errors::{db_error::DatabaseError, server_error::ServerError};

#[derive(Error, Debug)]
pub(crate) struct DatabaseErrorWrapper {
    pub error: yesser_todo_errors::db_error::DatabaseError,
}

impl std::fmt::Display for DatabaseErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}

impl From<yesser_todo_db::DatabaseError> for DatabaseErrorWrapper {
    fn from(value: yesser_todo_db::DatabaseError) -> Self {
        match value {
            yesser_todo_db::DatabaseError::JsonError(error) => DatabaseErrorWrapper {
                error: DatabaseError::JsonError(error),
            },
            yesser_todo_db::DatabaseError::IOError(error) => DatabaseErrorWrapper {
                error: DatabaseError::IOError(error),
            },
            yesser_todo_db::DatabaseError::UserDirsError => DatabaseErrorWrapper {
                error: DatabaseError::UserDirsError,
            },
        }
    }
}

impl From<DatabaseErrorWrapper> for ServerError {
    fn from(value: DatabaseErrorWrapper) -> Self {
        value.error.into()
    }
}
