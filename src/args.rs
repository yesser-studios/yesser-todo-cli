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
    Add(TasksCommand),
    /// Remove tasks from the task list. Separate tasks with spaces.
    Remove(TasksCommand),
    /// Mark tasks in the task list as done.
    Done(TasksCommand),
    /// Mark tasks in the list as undone.
    Undone(TasksCommand),
    /// List all tasks. Tasks marked done are shown with a strike-through.
    List
}

#[derive(Debug, Args)]
pub(crate) struct TasksCommand {
    /// The tasks to add/remove/mark done
    #[arg(num_args = 1..)]
    pub tasks: Vec<String>
}