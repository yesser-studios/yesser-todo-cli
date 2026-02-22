pub(crate) enum CommandError {
    NoTasksSpecified,
    TaskExists { name: String },
    TaskNotFound { name: String },
    DataError { what: String },
    HTTPError { name: String, status_code: u16 },
    ConnectionError { name: String },
    UnlinkedError,
}

impl CommandError {
    pub(crate) fn handle(&self) {
        match self {
            CommandError::NoTasksSpecified => {
                println!("No tasks specified!") // TODO: Change to eprintln! in next major version
            }
            CommandError::TaskExists { name } => {
                println!("Task {name} already exists!")
            }
            CommandError::TaskNotFound { name } => {
                println!("Task {name} not found!")
            }
            CommandError::DataError { what } => {
                println!("Unable to save {what}.")
            }
            CommandError::HTTPError { name, status_code } => match name.as_str() {
                "" => println!("HTTP error code {status_code}!"),
                _ => println!("HTTP error code {status_code} for task {name}!"),
            },
            CommandError::ConnectionError { name } => match name.as_str() {
                "" => println!("Failed to connect to the server!"),
                _ => println!("Failed to connect to the server for task {name}!"),
            },
            CommandError::UnlinkedError => {
                println!("You're already unlinked!");
            }
        }
    }
}
