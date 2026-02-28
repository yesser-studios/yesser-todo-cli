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




















        Self::IOError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_error_from() {
        let json_str = "invalid json";
        let serde_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let db_err = DatabaseError::from(serde_err);
        assert!(matches!(db_err, DatabaseError::JsonError(_)));
    }

    #[test]
    fn test_json_error_display() {
        let json_str = "invalid json";
        let serde_err = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let db_err = DatabaseError::JsonError(serde_err);
        let msg = format!("{}", db_err);
        assert!(msg.contains("JSON deserialization error"));
    }

    #[test]
    fn test_io_error_from() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let db_err = DatabaseError::from(io_err);
        assert!(matches!(db_err, DatabaseError::IOError(_)));
    }

    #[test]
    fn test_io_error_display() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let db_err = DatabaseError::IOError(io_err);
        let msg = format!("{}", db_err);
        assert!(msg.contains("error occurred while writing to config file"));
    }

    #[test]
    fn test_user_dirs_error_display() {
        let db_err = DatabaseError::UserDirsError;
        let msg = format!("{}", db_err);
        assert_eq!(msg, "Could not get user config directory location");
    }

    #[test]
    fn test_error_trait() {
        let err = DatabaseError::UserDirsError;
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn test_debug_impl() {
        let err = DatabaseError::UserDirsError;
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("UserDirsError"));
    }

    #[test]
    fn test_io_error_kinds() {
        let kinds = vec![
            std::io::ErrorKind::NotFound,
            std::io::ErrorKind::PermissionDenied,
            std::io::ErrorKind::ConnectionRefused,
            std::io::ErrorKind::AlreadyExists,
        ];
        for kind in kinds {
            let io_err = std::io::Error::new(kind, "test error");
            let db_err = DatabaseError::from(io_err);
            assert!(matches!(db_err, DatabaseError::IOError(_)));
        }
    }

    #[test]
    fn test_json_error_chain() {
        let json_str = "{invalid}";
        let result: Result<serde_json::Value, DatabaseError> = serde_json::from_str(json_str).map_err(DatabaseError::from);
        assert!(result.is_err());
        match result.unwrap_err() {
            DatabaseError::JsonError(_) => {},
            _ => panic!("expected JsonError"),
        }
    }
}