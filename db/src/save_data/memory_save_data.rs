use yesser_todo_errors::db_error::DatabaseError;

use crate::{CloudConfig, SaveData};
use std::{cell::RefCell, ops::Deref};

use crate::Task;

pub struct MemorySaveData {
    tasks: Vec<Task>,
    cloud_config: Option<CloudConfig>,
}

impl MemorySaveData {
    /// Constructs an empty `MemorySaveData`.
    ///
    /// # Returns
    ///
    /// A `MemorySaveData` whose internal task list is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::{save_data::MemorySaveData, SaveData};
    /// let mut data = MemorySaveData::new();
    /// assert!(data.get_tasks().is_empty());
    /// ```
    pub fn new() -> MemorySaveData {
        MemorySaveData {
            tasks: Vec::new(),
            cloud_config: None,
        }
    }

    pub fn new_with_cloud_config(cloud_config: CloudConfig) -> MemorySaveData {
        MemorySaveData {
            tasks: Vec::new(),
            cloud_config: Some(cloud_config),
        }
    }
}

impl Default for MemorySaveData {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveData for MemorySaveData {
    /// Retrieves the saved cloud configuration, if present.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::{save_data::MemorySaveData, SaveData};
    ///
    /// // This example assumes no cloud config is present or a valid one exists.
    /// let data = MemorySaveData::new();
    /// let res = data.get_cloud_config();
    ///
    /// match res {
    ///     Ok(None) => println!("No cloud config saved"),
    ///     Ok(Some((host, port))) => println!("Cloud config: {}:{}", host, port),
    ///     Err(e) => panic!("Failed to read cloud config: {}", e),
    /// }
    /// ```
    fn get_cloud_config(&self) -> Result<Option<(String, String)>, DatabaseError> {
        match &self.cloud_config {
            None => Ok(None),
            Some(cloud_config) => Ok(Some((cloud_config.host.clone(), cloud_config.port.clone()))),
        }
    }

    /// Stores the cloud host and port.
    ///
    /// # Parameters
    ///
    /// - `host`: cloud server host name or address.
    /// - `port`: cloud server port.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use yesser_todo_db::{save_data::MemorySaveData, SaveData};
    ///
    /// let mut data = MemorySaveData::new();
    /// data.save_cloud_config("example.com", "1234");
    /// ```
    fn save_cloud_config(&mut self, host: &str, port: &str) -> Result<(), DatabaseError> {
        self.cloud_config = Some(CloudConfig {
            host: host.to_string(),
            port: port.to_string(),
        });
        Ok(())
    }

    /// Remove the cloud configuration file from the application's config directory.
    ///
    /// # Errors
    ///
    /// Returns a `DatabaseError` if the file cannot be removed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use yesser_todo_db::{save_data::MemorySaveData, SaveData};
    ///
    /// let mut data = MemorySaveData::new();
    /// // Attempt to remove the cloud configuration file.
    /// let _ = data.remove_cloud_config();
    /// ```
    fn remove_cloud_config(&mut self) -> Result<(), DatabaseError> {
        self.cloud_config = None;
        Ok(())
    }

    /// Does nothing, as all tasks are stored in memory.
    ///
    /// # Returns
    ///
    /// `Ok(())`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use yesser_todo_db::{save_data::MemorySaveData, SaveData};
    /// let mut data = MemorySaveData::new();
    /// // If no data file is present this will succeed and leave tasks empty.
    /// data.load_tasks();
    /// assert!(data.get_tasks().is_empty());
    /// ```
    fn load_tasks(&mut self) -> Result<(), DatabaseError> {
        Ok(())
    }

    /// Does nothing, as all tasks are stored in memory.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use yesser_todo_db::{save_data::MemorySaveData, SaveData};
    ///
    /// let data = MemorySaveData::new();
    /// data.save_tasks();
    /// ```
    fn save_tasks(&self) -> Result<(), DatabaseError> {
        Ok(())
    }

    /// Access the internal list of tasks for in-place modification.
    ///
    /// Returns a mutable reference to the internal `Vec<Task>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::{save_data::MemorySaveData, SaveData, Task};
    /// let mut data = MemorySaveData::new();
    /// data.get_tasks().push(Task { name: "buy milk".to_string(), done: false });
    /// assert_eq!(data.get_tasks().len(), 1);
    /// data.get_tasks()[0].done = true;
    /// assert!(data.get_tasks()[0].done);
    /// ```
    fn get_tasks(&mut self) -> &mut Vec<Task> {
        &mut self.tasks
    }

    /// Appends the given task to the internal list of tasks.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::{save_data::MemorySaveData, SaveData, Task};
    /// let mut data = MemorySaveData::new();
    /// data.add_task(Task { name: "Write tests".into(), done: false });
    /// assert_eq!(data.get_tasks().len(), 1);
    /// ```
    fn add_task(&mut self, task: Task) {
        self.tasks.push(task)
    }

    /// Removes the task at the given index from the internal list of tasks.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::{save_data::MemorySaveData, SaveData, Task};
    /// let mut data = MemorySaveData::new();
    /// data.add_task(Task { name: "Write tests".into(), done: false });
    /// data.remove_task(0);
    /// assert_eq!(data.get_tasks().len(), 0);
    /// ```
    fn remove_task(&mut self, task_index: usize) {
        self.tasks.remove(task_index);
    }

    /// Mark the task at the given index as done and return its previous completion state.
    ///
    /// # Parameters
    ///
    /// - `task_index`: Index of the task within the internal tasks list.
    ///
    /// # Panics
    ///
    /// Panics if `task_index` is out of bounds for the tasks list.
    ///
    /// # Returns
    ///
    /// `true` if the task was *already done* before marking as done, `false` if it was done.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::{save_data::MemorySaveData, SaveData, Task};
    /// let mut data = MemorySaveData::new();
    /// data.add_task(Task { name: "a".into(), done: false });
    /// // It wasn't done, so returns false.
    /// assert_eq!(data.mark_task_done(0), false);
    /// assert_eq!(data.get_tasks()[0].done, true);
    /// // Now it's done, so returns true
    /// assert_eq!(data.mark_task_done(0), true);
    /// assert_eq!(data.get_tasks()[0].done, true);
    /// ```
    /// Notice how the task is still done after the second call. Use `mark_task_undone` to undo this.
    fn mark_task_done(&mut self, task_index: usize) -> bool {
        let was_done = self.tasks[task_index].done;
        self.tasks[task_index].done = true;
        was_done
    }

    /// Marks the task at the given index as not done and returns whether it was already not done.
    ///
    /// # Panics
    ///
    /// Panics if `task_index` is out of bounds for the tasks list.
    ///
    /// # Returns
    ///
    /// `true` if the task was *already not done* before marking as undone, `false` if it was done.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::{save_data::MemorySaveData, SaveData, Task};
    /// let mut data = MemorySaveData::new();
    /// data.add_task(Task { name: "task".into(), done: true });
    /// // It was done, returns false.
    /// assert_eq!(data.mark_task_undone(0), false);
    /// assert_eq!(data.get_tasks()[0].done, false);
    /// // Now it's not done, so returns true.
    /// assert_eq!(data.mark_task_undone(0), true);
    /// assert_eq!(data.get_tasks()[0].done, false);
    /// ```
    /// Notice how the task is still undone after the second call. Use `mark_task_done` to undo this.
    fn mark_task_undone(&mut self, task_index: usize) -> bool {
        let was_undone = !self.tasks[task_index].done;
        self.tasks[task_index].done = false;
        was_undone
    }

    /// Removes all tasks from the saved task list.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::{save_data::MemorySaveData, SaveData, Task};
    /// let mut data = MemorySaveData::new();
    /// data.add_task(Task { name: "a".into(), done: false });
    /// data.add_task(Task { name: "b".into(), done: true });
    /// data.clear_tasks();
    /// assert!(data.get_tasks().is_empty());
    /// ```
    fn clear_tasks(&mut self) {
        self.tasks.clear();
    }

    /// Removes all tasks marked as completed from the internal task list.
    ///
    /// This keeps only tasks whose `done` field is `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::{save_data::MemorySaveData, SaveData, Task};
    /// let mut data = MemorySaveData::new();
    /// data.add_task(Task { name: "a".into(), done: false });
    /// data.add_task(Task { name: "b".into(), done: true });
    /// data.clear_done_tasks();
    /// let tasks = data.get_tasks();
    /// assert_eq!(tasks.len(), 1);
    /// assert_eq!(tasks[0].name, "a");
    /// ```
    fn clear_done_tasks(&mut self) {
        self.tasks.retain(|t| !t.done);
    }
}

#[cfg(test)]
mod tests {
    use crate::{SaveData, Task, save_data::memory_save_data::MemorySaveData};

    #[test]
    fn test_memory_save_data_new() {
        let save_data = MemorySaveData::new();
        assert_eq!(save_data.tasks.len(), 0);
    }

    #[test]
    fn test_save_data_add_task() {
        let mut save_data = MemorySaveData::new();
        let task = Task {
            name: "test task".to_string(),
            done: false,
        };
        save_data.add_task(task.clone());
        assert_eq!(save_data.tasks.len(), 1);
        assert_eq!(save_data.tasks[0], task);
    }

    #[test]
    fn test_save_data_add_multiple_tasks() {
        let mut save_data = MemorySaveData::new();
        save_data.add_task(Task {
            name: "task1".to_string(),
            done: false,
        });
        save_data.add_task(Task {
            name: "task2".to_string(),
            done: true,
        });
        save_data.add_task(Task {
            name: "task3".to_string(),
            done: false,
        });
        assert_eq!(save_data.tasks.len(), 3);
        assert_eq!(save_data.tasks[1].name, "task2");
    }

    #[test]
    fn test_save_data_get_tasks() {
        let mut save_data = MemorySaveData::new();
        save_data.add_task(Task {
            name: "test".to_string(),
            done: false,
        });
        let tasks = save_data.get_tasks();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "test");
    }

    #[test]
    fn test_save_data_get_tasks_mutable() {
        let mut save_data = MemorySaveData::new();
        save_data.add_task(Task {
            name: "test".to_string(),
            done: false,
        });
        let tasks = save_data.get_tasks();
        tasks[0].done = true;
        assert!(save_data.tasks[0].done);
    }

    #[test]
    fn test_save_data_remove_task() {
        let mut save_data = MemorySaveData::new();
        save_data.add_task(Task {
            name: "task1".to_string(),
            done: false,
        });
        save_data.add_task(Task {
            name: "task2".to_string(),
            done: false,
        });
        save_data.add_task(Task {
            name: "task3".to_string(),
            done: false,
        });
        save_data.remove_task(1);
        assert_eq!(save_data.tasks.len(), 2);
        assert_eq!(save_data.tasks[0].name, "task1");
        assert_eq!(save_data.tasks[1].name, "task3");
    }

    #[test]
    fn test_save_data_mark_task_done() {
        let mut save_data = MemorySaveData::new();
        save_data.add_task(Task {
            name: "task".to_string(),
            done: false,
        });
        let was_done = save_data.mark_task_done(0);
        assert!(!was_done);
        assert!(save_data.tasks[0].done);
    }

    #[test]
    fn test_save_data_mark_task_done_already_done() {
        let mut save_data = MemorySaveData::new();
        save_data.add_task(Task {
            name: "task".to_string(),
            done: true,
        });
        let was_done = save_data.mark_task_done(0);
        assert!(was_done);
        assert!(save_data.tasks[0].done);
    }

    #[test]
    fn test_save_data_mark_task_undone() {
        let mut save_data = MemorySaveData::new();
        save_data.add_task(Task {
            name: "task".to_string(),
            done: true,
        });
        let was_undone = save_data.mark_task_undone(0);
        assert!(!was_undone);
        assert!(!save_data.tasks[0].done);
    }

    #[test]
    fn test_save_data_mark_task_undone_already_undone() {
        let mut save_data = MemorySaveData::new();
        save_data.add_task(Task {
            name: "task".to_string(),
            done: false,
        });
        let was_undone = save_data.mark_task_undone(0);
        assert!(was_undone);
        assert!(!save_data.tasks[0].done);
    }

    #[test]
    fn test_save_data_clear_tasks() {
        let mut save_data = MemorySaveData::new();
        save_data.add_task(Task {
            name: "task1".to_string(),
            done: false,
        });
        save_data.add_task(Task {
            name: "task2".to_string(),
            done: true,
        });
        assert_eq!(save_data.tasks.len(), 2);
        save_data.clear_tasks();
        assert_eq!(save_data.tasks.len(), 0);
    }

    #[test]
    fn test_save_data_clear_done_tasks() {
        let mut save_data = MemorySaveData::new();
        save_data.add_task(Task {
            name: "undone1".to_string(),
            done: false,
        });
        save_data.add_task(Task {
            name: "done1".to_string(),
            done: true,
        });
        save_data.add_task(Task {
            name: "undone2".to_string(),
            done: false,
        });
        save_data.add_task(Task {
            name: "done2".to_string(),
            done: true,
        });
        save_data.clear_done_tasks();
        assert_eq!(save_data.tasks.len(), 2);
        assert_eq!(save_data.tasks[0].name, "undone1");
        assert_eq!(save_data.tasks[1].name, "undone2");
    }

    #[test]
    fn test_save_data_clear_done_tasks_no_done() {
        let mut save_data = MemorySaveData::new();
        save_data.add_task(Task {
            name: "task1".to_string(),
            done: false,
        });
        save_data.add_task(Task {
            name: "task2".to_string(),
            done: false,
        });
        save_data.clear_done_tasks();
        assert_eq!(save_data.tasks.len(), 2);
    }

    #[test]
    fn test_save_data_clear_done_tasks_all_done() {
        let mut save_data = MemorySaveData::new();
        save_data.add_task(Task {
            name: "task1".to_string(),
            done: true,
        });
        save_data.add_task(Task {
            name: "task2".to_string(),
            done: true,
        });
        save_data.clear_done_tasks();
        assert_eq!(save_data.tasks.len(), 0);
    }

    #[test]
    fn test_multiple_operations() {
        let mut save_data = MemorySaveData::new();
        save_data.add_task(Task {
            name: "task1".to_string(),
            done: false,
        });
        save_data.add_task(Task {
            name: "task2".to_string(),
            done: false,
        });
        save_data.mark_task_done(0);
        save_data.add_task(Task {
            name: "task3".to_string(),
            done: false,
        });
        save_data.remove_task(1);
        assert_eq!(save_data.tasks.len(), 2);
        assert!(save_data.tasks[0].done);
        assert_eq!(save_data.tasks[0].name, "task1");
        assert_eq!(save_data.tasks[1].name, "task3");
    }

    #[test]
    fn test_load_save_cloud_config() {
        let mut data = MemorySaveData::new();
        data.save_cloud_config("example.com", "6982");
        let result = data.get_cloud_config().unwrap();
        assert_eq!(result, Some(("example.com".to_string(), "6982".to_string())));
    }
}
