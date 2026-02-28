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
    /// // Assuming `DatabaseError` is in scope:
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
    /// use db::db_error::DatabaseError;
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
