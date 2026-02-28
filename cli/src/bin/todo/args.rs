use clap::{Args, Parser, Subcommand};
use yesser_todo_api::Client;
use yesser_todo_db::Task;

use crate::command_error::CommandError::UnlinkedError;
use crate::command_impl::*;
use crate::command_impl_cloud::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct TodoArgs {
    /// The operation to do in the task list.
    #[clap(subcommand)]
    pub(crate) command: Command,
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
    /// Remove all tasks (or specify --done to clear only done tasks). Please note that this is irreversible.
    Clear(ClearCommand),
    /// Remove all done tasks. Please note that this is irreversible.
    ClearDone,
    /// List all tasks. Tasks marked done are shown with a strike-through.
    List,
    /// Add connection configuration to a server.
    Connect(CloudCommand),
    /// Remove server configuration.
    Disconnect,
}

#[derive(Debug, Args)]
pub(crate) struct TasksCommand {
    /// The tasks to add/remove/mark done
    #[arg(num_args = 1..)]
    pub tasks: Vec<String>,
}

#[derive(Debug, Args)]
pub(crate) struct ClearCommand {
    /// Only clear done tasks
    #[arg(short, long)]
    pub done: bool,
}

#[derive(Debug, Args)]
pub(crate) struct CloudCommand {
    pub host: String,
    pub port: Option<String>,
}

impl Command {
    /// Execute the command against either a local task list or a connected cloud client.
    ///
    /// When `client` is `None`, the command operates on the provided mutable `data` (local handlers).
    /// When `client` is `Some`, the command is routed to the cloud client (cloud handlers). Connect and
    /// Disconnect are handled without requiring an active client; Disconnect returns an `UnlinkedError`
    /// if no client is connected.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a `crate::command_error::CommandError` on failure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use yesser_todo_db::Task;
    /// # use yesser_todo_api::Client;
    /// # use crate::cli::args::Command;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut tasks: Vec<Task> = Vec::new();
    /// let mut client: Option<Client> = None;
    /// let cmd = Command::List;
    /// cmd.execute(&mut tasks, &mut client).await?;
    /// # Ok(()) }
    /// ```
    pub(crate) async fn execute(&self, data: &mut Vec<Task>, client: &mut Option<Client>) -> Result<(), crate::command_error::CommandError> {
        match client {
            None => match self {
                Command::Add(tasks_command) => handle_add(tasks_command, data),
                Command::Remove(tasks_command) => handle_remove(tasks_command, data),
                Command::Done(tasks_command) => handle_done_undone(tasks_command, data, true),
                Command::Undone(tasks_command) => handle_done_undone(tasks_command, data, false),
                Command::Clear(clear_command) => handle_clear(clear_command, data),
                Command::ClearDone => handle_clear_done(data),
                Command::List => handle_list(data),
                Command::Connect(cloud_command) => handle_connect(cloud_command),
                Command::Disconnect => handle_disconnect(),
            },
            Some(client) => match self {
                Command::Add(tasks_command) => handle_add_cloud(tasks_command, client).await,
                Command::Remove(tasks_command) => handle_remove_cloud(tasks_command, client).await,
                Command::Done(tasks_command) => handle_done_undone_cloud(tasks_command, client, true).await,
                Command::Undone(tasks_command) => handle_done_undone_cloud(tasks_command, client, false).await,
                Command::Clear(clear_command) => handle_clear_cloud(clear_command, client).await,
                Command::ClearDone => handle_clear_done_cloud(client).await,
                Command::List => handle_list_cloud(client).await,
                Command::Connect(cloud_command) => handle_connect(cloud_command),
                Command::Disconnect => handle_disconnect(),
            },
        }
    }
}
