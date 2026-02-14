pub(crate) enum CommandError {
    TaskExists { name: String },
    TaskNotFound { name: String },
    HTTPError { name: String, status_code: u16 },
    ConnectionError { name: String },
}
