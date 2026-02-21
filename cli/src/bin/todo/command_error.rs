pub(crate) enum CommandError {
    NoTasksSpecified,
    TaskExists { name: String },
    TaskNotFound { name: String },
    HTTPError { name: String, status_code: u16 },
    ConnectionError { name: String },
}

impl CommandError {
    pub(crate) fn handle(&self) {
        match self {
            CommandError::NoTasksSpecified => {
                println!("No tasks specified!") // TODO: Change to eprintln! in next major version
            }
            CommandError::TaskExists { name } => {
                eprintln!("Task {name} already exists!")
            }
            CommandError::TaskNotFound { name } => {
                eprintln!("Task {name} not found!")
            }
            CommandError::HTTPError { name, status_code } => {
                eprintln!("HTTP error code {status_code} for task {name}!")
            }
            CommandError::ConnectionError { name } => {
                eprintln!("Failed to connect to the server for task {name}!")
            }
        }
    }
}
