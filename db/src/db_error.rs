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
    /// Convert a `serde_json::Error` into `DatabaseError::JsonError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::db_error::DatabaseError;
    /// let serde_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
    /// let db_err = DatabaseError::from(serde_err);
    /// assert!(matches!(db_err, DatabaseError::JsonError(_)));
    /// ```
    fn from(value: serde_json::Error) -> Self {
        Self::JsonError(value)
    }
}

impl From<std::io::Error> for DatabaseError {
    /// Convert an `std::io::Error` into a `DatabaseError::IOError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::db_error::DatabaseError;
    /// let io_err = std::io::Error::new(std::io::ErrorKind::Other, "oops");
    /// let db_err = DatabaseError::from(io_err);
    /// match db_err {
    ///     DatabaseError::IOError(e) => assert_eq!(e.to_string(), "oops"),
    ///     _ => panic!("expected IOError"),
    /// }
    /// ```
    fn from(err: std::io::Error) -> Self {
        Self::IOError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    #[test]
    fn test_from_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("not valid json").unwrap_err();
        let db_err = DatabaseError::from(json_err);
        assert!(matches!(db_err, DatabaseError::JsonError(_)));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(ErrorKind::NotFound, "file not found");
        let db_err = DatabaseError::from(io_err);
        match db_err {
            DatabaseError::IOError(e) => {
                assert_eq!(e.kind(), ErrorKind::NotFound);
                assert_eq!(e.to_string(), "file not found");
            }
            _ => panic!("expected IOError"),
        }
    }

    #[test]
    fn test_json_error_display() {
        let json_err = serde_json::from_str::<serde_json::Value>("{bad json}").unwrap_err();
        let db_err = DatabaseError::JsonError(json_err);
        let display = format!("{}", db_err);
        assert!(display.contains("JSON deserialization error occured"));
    }

    #[test]
    fn test_io_error_display() {
        let io_err = std::io::Error::new(ErrorKind::PermissionDenied, "access denied");
        let db_err = DatabaseError::IOError(io_err);
        let display = format!("{}", db_err);
        assert!(display.contains("error occurred while writing to config file or directory"));
        assert!(display.contains("access denied"));
    }

    #[test]
    fn test_user_dirs_error_display() {
        let db_err = DatabaseError::UserDirsError;
        let display = format!("{}", db_err);
        assert_eq!(display, "Could not get user config directory location");
    }

    #[test]
    fn test_database_error_is_error_trait() {
        fn assert_error<T: std::error::Error>() {}
        assert_error::<DatabaseError>();
    }

    #[test]
    fn test_database_error_debug() {
        let db_err = DatabaseError::UserDirsError;
        let debug_str = format!("{:?}", db_err);
        assert!(debug_str.contains("UserDirsError"));
    }

    #[test]
    fn test_from_various_io_error_kinds() {
        let error_kinds = vec![
            ErrorKind::NotFound,
            ErrorKind::PermissionDenied,
            ErrorKind::AlreadyExists,
            ErrorKind::WouldBlock,
        ];

        for kind in error_kinds {
            let io_err = std::io::Error::new(kind, "test error");
            let db_err = DatabaseError::from(io_err);
            match db_err {
                DatabaseError::IOError(e) => assert_eq!(e.kind(), kind),
                _ => panic!("expected IOError for kind {:?}", kind),
            }
        }
    }

    #[test]
    fn test_json_error_from_invalid_types() {
        let json_err = serde_json::from_str::<Vec<String>>("{\"not\": \"an array\"}").unwrap_err();
        let db_err = DatabaseError::from(json_err);
        assert!(matches!(db_err, DatabaseError::JsonError(_)));
    }
}