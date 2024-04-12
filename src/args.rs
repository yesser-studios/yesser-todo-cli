use clap:: {
    Args,
    Parser,
    Subcommand
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct TodoArgs {
    /// The operation to do in the task list.
    #[clap(subcommand)]
    pub(crate) command: Command
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Add tasks to the task list. Separate tasks with spaces.
    Add(AddCommand),
    /// Remove tasks from the task list. Separate tasks with spaces.
    Remove(RemoveCommand),
    /// Mark tasks in the task list as done.
    Done(DoneCommand),
    /// List all tasks. Tasks marked done are shown with a strike-through.
    List
}

#[derive(Debug, Args)]
pub(crate) struct AddCommand {
    /// The tasks to add
    #[arg(num_args = 0..)]
    pub tasks: Option<Vec<String>>
}

#[derive(Debug, Args)]
pub(crate) struct RemoveCommand {
    /// The tasks to remove
    #[arg(num_args = 0..)]
    pub tasks: Option<Vec<String>>
}

#[derive(Debug, Args)]
pub(crate) struct DoneCommand {
    /// The tasks to mark done
    #[arg(num_args = 0..)]
    pub tasks: Option<Vec<String>>
}