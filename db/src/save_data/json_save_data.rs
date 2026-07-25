use serde_json::{from_reader, to_writer};
use yesser_todo_errors::db_error::DatabaseError;

use crate::{CloudConfig, SaveData};
use std::{
    fs::{self, File},
    path::PathBuf,
};

use crate::Task;

pub struct JsonSaveData {
    tasks: Vec<Task>,
    data_dir: PathBuf,
    config_dir: PathBuf,
}

impl JsonSaveData {
    /// Constructs an empty `JsonSaveData` using platform-specific default directories.
    ///
    /// # Returns
    ///
    /// A `JsonSaveData` whose internal task list is empty.
    ///
    /// # Errors
    ///
    /// Returns `DatabaseError::UserDirsError` if platform-specific user directories cannot be determined.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::{save_data::JsonSaveData, SaveData};
    /// let mut data = JsonSaveData::new().unwrap();
    /// assert!(data.get_tasks().is_empty());
    /// ```
    pub fn new() -> Result<JsonSaveData, DatabaseError> {
        let app_dirs = platform_dirs::AppDirs::new(Some("todo"), true).ok_or(DatabaseError::UserDirsError)?;
        Ok(JsonSaveData {
            tasks: Vec::new(),
            data_dir: app_dirs.data_dir,
            config_dir: app_dirs.config_dir,
        })
    }

    /// Constructs a `JsonSaveData` with separate data and config directories.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use yesser_todo_db::{save_data::JsonSaveData, SaveData};
    ///
    /// let mut data = JsonSaveData::with_dirs(
    ///     PathBuf::from("/tmp/my-data"),
    ///     PathBuf::from("/tmp/my-config"),
    /// );
    /// assert!(data.get_tasks().is_empty());
    /// ```
    pub fn with_dirs(data_dir: PathBuf, config_dir: PathBuf) -> JsonSaveData {
        JsonSaveData {
            tasks: Vec::new(),
            data_dir,
            config_dir,
        }
    }

    /// Constructs a `JsonSaveData` using a single base directory for both data and config.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use yesser_todo_db::{save_data::JsonSaveData, SaveData};
    ///
    /// let mut data = JsonSaveData::with_dir(PathBuf::from("/tmp/my-base"));
    /// assert!(data.get_tasks().is_empty());
    /// ```
    pub fn with_dir(dir: PathBuf) -> JsonSaveData {
        JsonSaveData {
            tasks: Vec::new(),
            data_dir: dir.clone(),
            config_dir: dir,
        }
    }

    #[cfg(test)]
    pub fn new_temp() -> Result<(JsonSaveData, tempfile::TempDir), DatabaseError> {
        let dir = tempfile::tempdir().map_err(DatabaseError::IOError)?;
        let path = dir.path().to_owned();
        Ok((
            JsonSaveData {
                tasks: Vec::new(),
                data_dir: path.clone(),
                config_dir: path,
            },
            dir,
        ))
    }

    fn data_file_path(&self) -> PathBuf {
        self.data_dir.join("todos.json")
    }

    fn config_file_path(&self) -> PathBuf {
        self.config_dir.join("cloud.json")
    }
}

impl SaveData for JsonSaveData {
    /// Retrieves the saved cloud configuration, if present.
    ///
    /// If a cloud configuration file exists and contains valid JSON matching `CloudConfig`,
    /// returns `Some((host, port))`. If the configuration file is missing, returns `None`.
    /// I/O or deserialization failures are returned as `DatabaseError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::{save_data::JsonSaveData, SaveData};
    ///
    /// // This example assumes no cloud config is present or a valid one exists.
    /// let data = JsonSaveData::new().unwrap();
    /// let res = data.get_cloud_config();
    ///
    /// match res {
    ///     Ok(None) => println!("No cloud config saved"),
    ///     Ok(Some((host, port))) => println!("Cloud config: {}:{}", host, port),
    ///     Err(e) => panic!("Failed to read cloud config: {}", e),
    /// }
    /// ```
    fn get_cloud_config(&self) -> Result<Option<(String, String)>, DatabaseError> {
        let config_file_path = self.config_file_path();

        if !config_file_path.exists() {
            return Ok(None);
        }

        let file = File::open(config_file_path)?;
        let result: CloudConfig = from_reader(file)?;

        Ok(Some((result.host.clone(), result.port.clone())))
    }

    /// Writes the cloud host and port to the application's `cloud.json` in the config directory.
    ///
    /// Creates the config directory if it does not exist and overwrites any existing `cloud.json`.
    ///
    /// # Parameters
    ///
    /// - `host`: cloud server host name or address.
    /// - `port`: cloud server port.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(DatabaseError)` if file I/O or serialization fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use yesser_todo_db::{save_data::JsonSaveData, SaveData};
    ///
    /// let data = JsonSaveData::new().unwrap();
    /// data.save_cloud_config("example.com", "1234").unwrap();
    /// ```
    fn save_cloud_config(&self, host: &str, port: &str) -> Result<(), DatabaseError> {
        fs::create_dir_all(&self.config_dir)?;
        let file = File::create(self.config_file_path())?;
        to_writer(file, &CloudConfig::new(host, port))?;

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
    /// use yesser_todo_db::{save_data::JsonSaveData, SaveData};
    ///
    /// let data = JsonSaveData::new().unwrap();
    /// // Attempt to remove the cloud configuration file.
    /// let _ = data.remove_cloud_config();
    /// ```
    fn remove_cloud_config(&self) -> Result<(), DatabaseError> {
        fs::remove_file(self.config_file_path())?;
        Ok(())
    }

    /// Loads tasks from the application's data file into this `JsonSaveData` instance.
    ///
    /// Ensures the application's data directory exists; if the data file is missing, no changes
    /// are made to the existing tasks. When the data file is present it is deserialized and
    /// replaces the current task list.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a `DatabaseError` if directory creation, file I/O, or deserialization fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use yesser_todo_db::{save_data::JsonSaveData, SaveData};
    /// let mut data = JsonSaveData::new().unwrap();
    /// // If no data file is present this will succeed and leave tasks empty.
    /// data.load_tasks().unwrap();
    /// assert!(data.get_tasks().is_empty());
    /// ```
    fn load_tasks(&mut self) -> Result<(), DatabaseError> {
        let data_file_path = self.data_file_path();

        fs::create_dir_all(&self.data_dir)?;

        if !data_file_path.exists() {
            return Ok(());
        }

        let file = File::open(data_file_path)?;

        let result: Vec<Task> = from_reader(file)?;
        self.tasks = result;

        Ok(())
    }

    /// Writes the current task list to the platform-specific data file (todos.json), creating the data directory if needed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use yesser_todo_db::{save_data::JsonSaveData, SaveData};
    ///
    /// let data = JsonSaveData::new().unwrap();
    /// data.save_tasks().unwrap();
    /// ```
    fn save_tasks(&self) -> Result<(), DatabaseError> {
        fs::create_dir_all(&self.data_dir)?;

        let file = File::create(self.data_file_path())?;

        to_writer(file, &self.tasks)?;

        Ok(())
    }

    /// Access the internal list of tasks for in-place modification.
    ///
    /// Returns a mutable reference to the internal `Vec<Task>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::{save_data::JsonSaveData, SaveData, Task};
    /// let mut data = JsonSaveData::new().unwrap();
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
    /// use yesser_todo_db::{save_data::JsonSaveData, SaveData, Task};
    /// let mut data = JsonSaveData::new().unwrap();
    /// data.add_task(Task { name: "Write tests".into(), done: false });
    /// assert_eq!(data.get_tasks().len(), 1);
    /// ```
    fn add_task(&mut self, task: Task) {
        self.tasks.push(task)
    }

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
    /// `true` if the task was already marked done, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::{save_data::JsonSaveData, SaveData, Task};
    /// let mut data = JsonSaveData::new().unwrap();
    /// data.add_task(Task { name: "a".into(), done: false });
    /// let prev = data.mark_task_done(0);
    /// assert_eq!(prev, false);
    /// assert_eq!(data.get_tasks()[0].done, true);
    /// ```
    fn mark_task_done(&mut self, task_index: usize) -> bool {
        let was_done = self.tasks[task_index].done;
        self.tasks[task_index].done = true;
        was_done
    }

    /// Marks the task at the given index as not done and returns whether it was already not done.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::{save_data::JsonSaveData, SaveData, Task};
    /// let mut data = JsonSaveData::new().unwrap();
    /// data.add_task(Task { name: "task".into(), done: true });
    /// // It was done, so the previous "undone" state is false.
    /// assert_eq!(data.mark_task_undone(0), false);
    /// // Now it's already not done, so the previous "undone" state is true.
    /// assert_eq!(data.mark_task_undone(0), true);
    /// ```
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
    /// use yesser_todo_db::{save_data::JsonSaveData, SaveData, Task};
    /// let mut data = JsonSaveData::new().unwrap();
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
    /// use yesser_todo_db::{save_data::JsonSaveData, SaveData, Task};
    /// let mut data = JsonSaveData::new().unwrap();
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
    use crate::{SaveData, Task, save_data::json_save_data::JsonSaveData};

    #[test]
    fn test_json_save_data_new() {
        let (save_data, _dir) = JsonSaveData::new_temp().unwrap();
        assert_eq!(save_data.tasks.len(), 0);
    }

    #[test]
    fn test_json_save_data_with_dir() {
        let dir = tempfile::tempdir().unwrap();
        let save_data = JsonSaveData::with_dir(dir.path().to_owned());
        assert_eq!(save_data.tasks.len(), 0);
    }

    #[test]
    fn test_json_save_data_with_dirs() {
        let data_dir = tempfile::tempdir().unwrap();
        let config_dir = tempfile::tempdir().unwrap();
        let save_data = JsonSaveData::with_dirs(data_dir.path().to_owned(), config_dir.path().to_owned());
        assert_eq!(save_data.tasks.len(), 0);
    }

    #[test]
    fn test_save_data_add_task() {
        let (mut save_data, _dir) = JsonSaveData::new_temp().unwrap();
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
        let (mut save_data, _dir) = JsonSaveData::new_temp().unwrap();
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
        let (mut save_data, _dir) = JsonSaveData::new_temp().unwrap();
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
        let (mut save_data, _dir) = JsonSaveData::new_temp().unwrap();
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
        let (mut save_data, _dir) = JsonSaveData::new_temp().unwrap();
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
        let (mut save_data, _dir) = JsonSaveData::new_temp().unwrap();
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
        let (mut save_data, _dir) = JsonSaveData::new_temp().unwrap();
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
        let (mut save_data, _dir) = JsonSaveData::new_temp().unwrap();
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
        let (mut save_data, _dir) = JsonSaveData::new_temp().unwrap();
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
        let (mut save_data, _dir) = JsonSaveData::new_temp().unwrap();
        save_data.add_task(Task {
            name: "task1".to_string(),
            done: false,
        });
        save_data.add_task(Task {
            name: "task2".to_string(),
            done: true,
        });
        save_data.clear_tasks();
        assert_eq!(save_data.tasks.len(), 0);
    }

    #[test]
    fn test_save_data_clear_done_tasks() {
        let (mut save_data, _dir) = JsonSaveData::new_temp().unwrap();
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
        let (mut save_data, _dir) = JsonSaveData::new_temp().unwrap();
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
        let (mut save_data, _dir) = JsonSaveData::new_temp().unwrap();
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
        let (mut save_data, _dir) = JsonSaveData::new_temp().unwrap();
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
        let (data, _dir) = JsonSaveData::new_temp().unwrap();
        data.save_cloud_config("example.com", "6982").unwrap();
        let result = data.get_cloud_config().unwrap();
        assert_eq!(result, Some(("example.com".to_string(), "6982".to_string())));
    }

    #[test]
    fn test_remove_nonexistent_cloud_config() {
        let (data, _dir) = JsonSaveData::new_temp().unwrap();
        assert!(data.get_cloud_config().unwrap().is_none());
        let result = data.remove_cloud_config();
        assert!(result.is_err());
    }

    #[test]
    fn test_load_save_tasks_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_owned();

        let mut data = JsonSaveData::with_dir(path.clone());
        data.add_task(Task {
            name: "task1".into(),
            done: false,
        });
        data.add_task(Task {
            name: "task2".into(),
            done: true,
        });
        data.save_tasks().unwrap();
        drop(data);

        let mut loaded = JsonSaveData::with_dir(path);
        loaded.load_tasks().unwrap();
        assert_eq!(loaded.get_tasks().len(), 2);
        assert_eq!(loaded.get_tasks()[0].name, "task1");
        assert!(!loaded.get_tasks()[0].done);
        assert_eq!(loaded.get_tasks()[1].name, "task2");
        assert!(loaded.get_tasks()[1].done);
    }

    #[test]
    fn test_persistence_with_dir() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_owned();

        let mut data = JsonSaveData::with_dir(path.clone());
        data.add_task(Task {
            name: "persist".into(),
            done: true,
        });
        data.save_tasks().unwrap();
        drop(data);

        let mut loaded = JsonSaveData::with_dir(path);
        loaded.load_tasks().unwrap();
        assert_eq!(loaded.get_tasks().len(), 1);
        assert_eq!(loaded.get_tasks()[0].name, "persist");
        assert!(loaded.get_tasks()[0].done);
    }
}
