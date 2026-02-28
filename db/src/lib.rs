pub mod db_error;

use std::{
    fs::{self, File},
    path::PathBuf,
};

use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_writer};

use crate::db_error::DatabaseError;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub name: String,
    pub done: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CloudConfig {
    pub host: String,
    pub port: String,
}

impl CloudConfig {
    /// Construct a `CloudConfig` that owns copies of the provided host and port strings.
    ///
    /// The returned `CloudConfig` contains owned `String` values cloned from the provided references.
    ///
    /// # Examples
    ///
    /// ```
    /// # use yesser_todo_db::CloudConfig;
    /// let host = "example.com".to_string();
    /// let port = "8080".to_string();
    /// let cfg = CloudConfig::new(&host, &port);
    /// assert_eq!(cfg.host, "example.com");
    /// assert_eq!(cfg.port, "8080");
    /// ```
    pub fn new(host: &str, port: &str) -> Self {
        CloudConfig {
            host: host.to_string(),
            port: port.to_string(),
        }
    }
}

pub struct SaveData {
    tasks: Vec<Task>,
}

/// Checks whether a task's name exactly equals a query string.
///
/// # Examples
///
/// ```
/// let task = Task { name: "Buy milk".into(), done: false };
/// assert!(exactly_matches(&task, "Buy milk"));
/// assert!(!exactly_matches(&task, "buy milk"));
/// ```
///
/// # Returns
///
/// `true` if the task's name equals `query_string`, `false` otherwise.
pub fn exactly_matches(task: &Task, query_string: &str) -> bool {
    return task.name == *query_string;
}

/// Finds the position of the first task whose name exactly matches the given query string.
///
/// # Returns
///
/// The zero-based index of the matching task, or `None` if no match is found.
///
/// # Examples
///
/// ```
/// # use crate::Task;
/// # use crate::get_index;
/// let tasks = vec![Task { name: "one".into(), done: false }, Task { name: "two".into(), done: true }];
/// assert_eq!(get_index(&tasks, "two"), Some(1));
/// assert_eq!(get_index(&tasks, "three"), None);
/// ```
pub fn get_index(tasks: &Vec<Task>, query_string: &str) -> Option<usize> {
    return tasks.iter().position(|r| exactly_matches(r, query_string));
}

impl SaveData {
    /// Constructs an empty SaveData.
    ///
    /// # Returns
    ///
    /// A `SaveData` whose internal task list is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut sd = SaveData::new();
    /// assert!(sd.get_tasks().is_empty());
    /// ```
    pub fn new() -> SaveData {
        return SaveData { tasks: Vec::new() };
    }

    /// Get the platform-specific `AppDirs` for the application and the full path to `todos.json` in the app data directory.
    ///
    /// # Errors
    ///
    /// Returns `DatabaseError::UserDirsError` if platform-specific application directories cannot be created.
    ///
    /// # Examples
    ///
    /// ```
    /// let (app_dirs, data_path) = yesser_todo_db::SaveData::get_data_paths().unwrap();
    /// assert!(data_path.starts_with(app_dirs.data_dir));
    /// assert_eq!(data_path.file_name().unwrap(), "todos.json");
    /// ```
    pub(crate) fn get_data_paths() -> Result<(AppDirs, PathBuf), DatabaseError> {
        let app_dirs = AppDirs::new(Some("todo"), true).ok_or_else(|| DatabaseError::UserDirsError)?;
        let data_file_path = app_dirs.data_dir.join("todos.json");
        return Ok((app_dirs, data_file_path));
    }

    /// Constructs the platform-specific application directories and the full path to the cloud config file.
    ///
    /// The returned PathBuf points to `cloud.json` inside the application's config directory.
    ///
    /// # Errors
    ///
    /// Returns `DatabaseError::UserDirsError` if platform-specific user directories cannot be determined.
    ///
    /// # Examples
    ///
    /// ```
    /// let Ok((_app_dirs, config_path)) = yesser_todo_db::SaveData::get_cloud_config_paths();
    /// assert!(config_path.ends_with("cloud.json"));
    /// ```
    pub(crate) fn get_cloud_config_paths() -> Result<(AppDirs, PathBuf), DatabaseError> {
        let app_dirs = AppDirs::new(Some("todo"), true).ok_or_else(|| DatabaseError::UserDirsError)?;
        let config_file_path = app_dirs.config_dir.join("cloud.json");
        return Ok((app_dirs, config_file_path));
    }

    /// Retrieves the saved cloud configuration, if present.
    ///
    /// If a cloud configuration file exists and contains valid JSON matching `CloudConfig`,
    /// returns `Some((host, port))`. If the configuration file is missing, returns `None`.
    /// I/O or deserialization failures are returned as `DatabaseError`.
    ///
    /// # Examples
    ///
    /// ```
    /// use yesser_todo_db::SaveData;
    ///
    /// // This example assumes no cloud config is present or a valid one exists.
    /// let res = SaveData::get_cloud_config();
    ///
    /// match res {
    ///     Ok(None) => println!("No cloud config saved"),
    ///     Ok(Some((host, port))) => println!("Cloud config: {}:{}", host, port),
    ///     Err(e) => panic!("Failed to read cloud config: {}", e),
    /// }
    /// ```
    pub fn get_cloud_config() -> Result<Option<(String, String)>, DatabaseError> {
        let config_paths = SaveData::get_cloud_config_paths()?;
        let app_dirs = config_paths.0;
        let config_file_path = config_paths.1;

        fs::create_dir_all(&app_dirs.config_dir)?;

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
    /// use yesser_todo_db::SaveData;
    ///
    /// SaveData::save_cloud_config("example.com", "1234").unwrap();
    /// ```
    pub fn save_cloud_config(host: &str, port: &str) -> Result<(), DatabaseError> {
        let config_paths = SaveData::get_cloud_config_paths()?;
        let app_dirs = config_paths.0;
        let config_file_path = config_paths.1;

        fs::create_dir_all(&app_dirs.config_dir)?;
        let file = File::create(config_file_path)?;
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
    /// use yesser_todo_db::SaveData;
    ///
    /// // Attempt to remove the cloud configuration file.
    /// let _ = SaveData::remove_cloud_config();
    /// ```
    pub fn remove_cloud_config() -> Result<(), DatabaseError> {
        let config_paths = SaveData::get_cloud_config_paths()?;
        let config_file_path = config_paths.1;

        fs::remove_file(config_file_path)?;
        Ok(())
    }

    /// Loads tasks from the application's data file into this `SaveData` instance.
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
    /// # use yesser_todo_db::SaveData;
    /// let mut sd = SaveData::new();
    /// // If no data file is present this will succeed and leave tasks empty.
    /// sd.load_tasks().unwrap();
    /// assert!(sd.get_tasks().is_empty());
    /// ```
    pub fn load_tasks(&mut self) -> Result<(), DatabaseError> {
        let data_paths = SaveData::get_data_paths()?;
        let app_dirs = data_paths.0;
        let data_file_path = data_paths.1;

        fs::create_dir_all(&app_dirs.data_dir)?;

        if !data_file_path.exists() {
            return Ok(());
        }

        let file = File::open(data_file_path)?;

        let result: Vec<Task> = from_reader(file)?;
        self.tasks = result;

        return Ok(());
    }

    /// Writes the current task list to the platform-specific data file (todos.json), creating the data directory if needed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use crate::SaveData;
    ///
    /// let data = SaveData::new();
    /// data.save_tasks().unwrap();
    /// ```
    pub fn save_tasks(&self) -> Result<(), DatabaseError> {
        let (app_dirs, data_file_path) = SaveData::get_data_paths()?;

        fs::create_dir_all(&app_dirs.data_dir)?;

        let file = File::create(data_file_path)?;

        to_writer(file, &self.tasks)?;

        return Ok(());
    }

    /// Access the internal list of tasks for in-place modification.
    ///
    /// Returns a mutable reference to the internal `Vec<Task>`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut data = SaveData::new();
    /// data.get_tasks().push(Task { name: "buy milk".to_string(), done: false });
    /// assert_eq!(data.get_tasks().len(), 1);
    /// data.get_tasks()[0].done = true;
    /// assert!(data.get_tasks()[0].done);
    /// ```
    pub fn get_tasks(&mut self) -> &mut Vec<Task> {
        return &mut self.tasks;
    }

    /// Appends the given task to the internal list of tasks.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut sd = SaveData::new();
    /// sd.add_task(Task { name: "Write tests".into(), done: false });
    /// assert_eq!(sd.get_tasks().len(), 1);
    /// ```
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task)
    }

    pub fn remove_task(&mut self, task_index: usize) {
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
    /// let mut data = SaveData::new();
    /// data.add_task(Task { name: "a".into(), done: false });
    /// let prev = data.mark_task_done(0);
    /// assert_eq!(prev, false);
    /// assert_eq!(data.get_tasks()[0].done, true);
    /// ```
    pub fn mark_task_done(&mut self, task_index: usize) -> bool {
        let was_done = self.tasks[task_index].done;
        self.tasks[task_index].done = true;
        return was_done;
    }

    /// Marks the task at the given index as not done and returns whether it was already not done.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut sd = SaveData::new();
    /// sd.add_task(Task { name: "task".into(), done: true });
    /// // It was done, so the previous "undone" state is false.
    /// assert_eq!(sd.mark_task_undone(0), false);
    /// // Now it's already not done, so the previous "undone" state is true.
    /// assert_eq!(sd.mark_task_undone(0), true);
    /// ```
    pub fn mark_task_undone(&mut self, task_index: usize) -> bool {
        let was_undone = !self.tasks[task_index].done;
        self.tasks[task_index].done = false;
        return was_undone;
    }

    /// Removes all tasks from the saved task list.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut sd = SaveData::new();
    /// sd.add_task(Task { name: "a".into(), done: false });
    /// sd.add_task(Task { name: "b".into(), done: true });
    /// sd.clear_tasks();
    /// assert!(sd.get_tasks().is_empty());
    /// ```
    pub fn clear_tasks(&mut self) {
        self.tasks.clear();
    }

    /// Removes all tasks marked as completed from the internal task list.
    ///
    /// This keeps only tasks whose `done` field is `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut store = SaveData::new();
    /// store.add_task(Task { name: "a".into(), done: false });
    /// store.add_task(Task { name: "b".into(), done: true });
    /// store.clear_done_tasks();
    /// let tasks = store.get_tasks();
    /// assert_eq!(tasks.len(), 1);
    /// assert_eq!(tasks[0].name, "a");
    /// ```
    pub fn clear_done_tasks(&mut self) {
        self.tasks.retain(|t| !t.done);
    }
}
