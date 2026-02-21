use clap::{Args, Parser, Subcommand};
use yesser_todo_api::Client;
use yesser_todo_db::{SaveData, Task};

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
    pub(crate) async fn execute(
        &self,
        data: &mut Vec<Task>,
        client: &mut Option<Client>,
    ) -> Result<(), crate::command_error::CommandError> {
        match client {
            None => match self {
                Command::Add(tasks_command) => handle_add(tasks_command, data),
                Command::Remove(tasks_command) => handle_remove(tasks_command, data),
                Command::Done(tasks_command) => todo!(),
                Command::Undone(tasks_command) => todo!(),
                Command::Clear(clear_command) => todo!(),
                Command::ClearDone => todo!(),
                Command::List => handle_list(data),
                Command::Connect(cloud_command) => todo!(),
                Command::Disconnect => todo!(),
            },
            Some(client) => match self {
                Command::Add(tasks_command) => handle_add_cloud(tasks_command, client).await,
                Command::Remove(tasks_command) => handle_remove_cloud(tasks_command, client).await,
                Command::Done(tasks_command) => todo!(),
                Command::Undone(tasks_command) => todo!(),
                Command::Clear(clear_command) => todo!(),
                Command::ClearDone => todo!(),
                Command::List => handle_list_cloud(client).await,
                Command::Connect(cloud_command) => todo!(),
                Command::Disconnect => todo!(),
            },
        }
    }
}
