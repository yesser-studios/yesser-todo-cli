pub(crate) enum CommandError {
    TaskExists { name: String },
    TaskNotFound,
}
