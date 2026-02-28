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
                Command::Disconnect => Err(UnlinkedError),
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






















        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_add_local() {
        let mut data = vec![];
        let mut client = None;
        let cmd = Command::Add(TasksCommand {
            tasks: vec!["test".to_string()],
        });
        let result = cmd.execute(&mut data, &mut client).await;
        assert!(result.is_ok());
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].name, "test");
    }

    #[tokio::test]
    async fn test_execute_remove_local() {
        let mut data = vec![Task { name: "test".to_string(), done: false }];
        let mut client = None;
        let cmd = Command::Remove(TasksCommand {
            tasks: vec!["test".to_string()],
        });
        let result = cmd.execute(&mut data, &mut client).await;
        assert!(result.is_ok());
        assert!(data.is_empty());
    }

    #[tokio::test]
    async fn test_execute_done_local() {
        let mut data = vec![Task { name: "test".to_string(), done: false }];
        let mut client = None;
        let cmd = Command::Done(TasksCommand {
            tasks: vec!["test".to_string()],
        });
        let result = cmd.execute(&mut data, &mut client).await;
        assert!(result.is_ok());
        assert!(data[0].done);
    }

    #[tokio::test]
    async fn test_execute_undone_local() {
        let mut data = vec![Task { name: "test".to_string(), done: true }];
        let mut client = None;
        let cmd = Command::Undone(TasksCommand {
            tasks: vec!["test".to_string()],
        });
        let result = cmd.execute(&mut data, &mut client).await;
        assert!(result.is_ok());
        assert!(!data[0].done);
    }

    #[tokio::test]
    async fn test_execute_clear_local() {
        let mut data = vec![
            Task { name: "test1".to_string(), done: false },
            Task { name: "test2".to_string(), done: true },
        ];
        let mut client = None;
        let cmd = Command::Clear(ClearCommand { done: false });
        let result = cmd.execute(&mut data, &mut client).await;
        assert!(result.is_ok());
        assert!(data.is_empty());
    }

    #[tokio::test]
    async fn test_execute_clear_done_local() {
        let mut data = vec![
            Task { name: "test1".to_string(), done: false },
            Task { name: "test2".to_string(), done: true },
        ];
        let mut client = None;
        let cmd = Command::Clear(ClearCommand { done: true });
        let result = cmd.execute(&mut data, &mut client).await;
        assert!(result.is_ok());
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].name, "test1");
    }

    #[tokio::test]
    async fn test_execute_list_local() {
        let mut data = vec![Task { name: "test".to_string(), done: false }];
        let mut client = None;
        let cmd = Command::List;
        let result = cmd.execute(&mut data, &mut client).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_disconnect_without_client() {
        let mut data = vec![];
        let mut client = None;
        let cmd = Command::Disconnect;
        let result = cmd.execute(&mut data, &mut client).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), crate::command_error::CommandError::UnlinkedError));
    }

    #[test]
    fn test_tasks_command_debug() {
        let cmd = TasksCommand {
            tasks: vec!["task1".to_string()],
        };
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("TasksCommand"));
    }

    #[test]
    fn test_clear_command_debug() {
        let cmd = ClearCommand { done: true };
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("ClearCommand"));
    }

    #[test]
    fn test_cloud_command_debug() {
        let cmd = CloudCommand {
            host: "example.com".to_string(),
            port: Some("8080".to_string()),
        };
        let debug_str = format!("{:?}", cmd);
        assert!(debug_str.contains("CloudCommand"));
    }

    #[test]
    fn test_command_enum_variants() {
        let add_cmd = Command::Add(TasksCommand { tasks: vec![] });
        let remove_cmd = Command::Remove(TasksCommand { tasks: vec![] });
        let done_cmd = Command::Done(TasksCommand { tasks: vec![] });
        let undone_cmd = Command::Undone(TasksCommand { tasks: vec![] });
        let clear_cmd = Command::Clear(ClearCommand { done: false });
        let clear_done_cmd = Command::ClearDone;
        let list_cmd = Command::List;
        let connect_cmd = Command::Connect(CloudCommand {
            host: "host".to_string(),
            port: None,
        });
        let disconnect_cmd = Command::Disconnect;

        assert!(format!("{:?}", add_cmd).contains("Add"));
        assert!(format!("{:?}", remove_cmd).contains("Remove"));
        assert!(format!("{:?}", done_cmd).contains("Done"));
        assert!(format!("{:?}", undone_cmd).contains("Undone"));
        assert!(format!("{:?}", clear_cmd).contains("Clear"));
        assert!(format!("{:?}", clear_done_cmd).contains("ClearDone"));
        assert!(format!("{:?}", list_cmd).contains("List"));
        assert!(format!("{:?}", connect_cmd).contains("Connect"));
        assert!(format!("{:?}", disconnect_cmd).contains("Disconnect"));
    }
}