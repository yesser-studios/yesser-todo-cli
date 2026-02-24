use std::{
    fs::{self, File},
    path::PathBuf,
};

use platform_dirs::AppDirs;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, to_writer};

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
    /// ```rust
    /// let host = "example.com".to_string();
    /// let port = "8080".to_string();
    /// let cfg = CloudConfig::new(&host, &port);
    /// assert_eq!(cfg.host, "example.com");
    /// assert_eq!(cfg.port, "8080");
    /// ```
    pub fn new(host: &String, port: &String) -> Self {
        CloudConfig {
            host: host.clone(),
            port: port.clone(),
        }
    }
}

pub struct SaveData {
    tasks: Vec<Task>,
}

pub fn exactly_matches(task: &Task, query_string: &String) -> bool {
    return task.name == *query_string;
}

pub fn get_index(tasks: &Vec<Task>, query_string: &String) -> Option<usize> {
    return tasks.iter().position(|r| exactly_matches(r, query_string));
}

impl SaveData {
    pub fn new() -> SaveData {
        return SaveData { tasks: Vec::new() };
    }

    /// Builds the application's platform-specific directories and the full path to the todos.json data file.
    ///
    /// # Returns
    ///
    /// A tuple containing the `AppDirs` for the application and a `PathBuf` pointing to `todos.json` inside the app's data directory.
    ///
    /// # Examples
    ///
    /// ```
    /// let (app_dirs, data_path) = db::get_data_paths();
    /// assert!(data_path.starts_with(app_dirs.data_dir));
    /// assert_eq!(data_path.file_name().unwrap(), "todos.json");
    /// ```
    pub(crate) fn get_data_paths() -> (AppDirs, PathBuf) {
        let app_dirs = AppDirs::new(Some("todo"), true).unwrap();
        let data_file_path = app_dirs.data_dir.join("todos.json");
        return (app_dirs, data_file_path);
    }

    /// Constructs application directories for this app and returns them together with the full path to the cloud config file.
    ///
    /// The returned path points to `cloud.json` located inside the application's config directory.
    ///
    /// # Examples
    ///
    /// ```
    /// let (_app_dirs, config_path) = db::get_cloud_config_paths();
    /// assert!(config_path.ends_with("cloud.json"));
    /// ```
    pub(crate) fn get_cloud_config_paths() -> (AppDirs, PathBuf) {
        let app_dirs = AppDirs::new(Some("todo"), true).unwrap();
        let config_file_path = app_dirs.config_dir.join("cloud.json");
        return (app_dirs, config_file_path);
    }

    /// Retrieves the cloud configuration if one has been saved.
    ///
    /// Ensures the application's config directory exists; if the cloud config file does not exist, returns `Ok(None)`. If the file exists and contains valid JSON matching `CloudConfig`, returns `Ok(Some((host, port)))`. Returns an `Err` if deserializing the file fails.
    ///
    /// # Examples
    ///
    /// ```
    /// // This example assumes no cloud config is present or a valid one exists.
    /// let res = SaveData::get_cloud_config();
    /// match res {
    ///     Ok(None) => println!("No cloud config saved"),
    ///     Ok(Some((host, port))) => println!("Cloud config: {}:{}", host, port),
    ///     Err(e) => panic!("Failed to read cloud config: {}", e),
    /// }
    /// ```
    pub fn get_cloud_config() -> Result<Option<(String, String)>, serde_json::Error> {
        let config_paths = SaveData::get_cloud_config_paths();
        let app_dirs = config_paths.0;
        let config_file_path = config_paths.1;

        fs::create_dir_all(&app_dirs.config_dir).unwrap();

        if !config_file_path.exists() {
            return Ok(None);
        }

        let file = File::open(config_file_path).unwrap();
        let result: CloudConfig = from_reader(file)?;

        Ok(Some((result.host.clone(), result.port.clone())))
    }

    /// Writes the given cloud host and port into the application's `cloud.json` inside the config directory.
    ///
    /// # Parameters
    ///
    /// - `host`: cloud server host name or address.
    /// - `port`: cloud server port.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(serde_json::Error)` if serialization or writing fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use db::SaveData;
    ///
    /// let host = "example.com".to_string();
    /// let port = "1234".to_string();
    /// SaveData::save_cloud_config(&host, &port).unwrap();
    /// ```
    pub fn save_cloud_config(host: &String, port: &String) -> Result<(), serde_json::Error> {
        let config_paths = SaveData::get_cloud_config_paths();
        let app_dirs = config_paths.0;
        let config_file_path = config_paths.1;

        fs::create_dir_all(&app_dirs.config_dir).unwrap();
        let file = File::create(config_file_path).unwrap();
        to_writer(file, &CloudConfig::new(host, port))?;

        Ok(())
    }

    /// Removes the cloud configuration file (`cloud.json`) from the application's config directory.
    ///
    /// The file path is obtained from `SaveData::get_cloud_config_paths()`. If removal fails, the underlying I/O error is returned.
    ///
    /// # Errors
    ///
    /// Returns the `std::io::Error` produced by `std::fs::remove_file` when the file cannot be removed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// // Remove the cloud config and handle any error.
    /// let result = SaveData::remove_cloud_config();
    /// if let Err(e) = result {
    ///     eprintln!("failed to remove cloud config: {}", e);
    /// }
    /// ```
    pub fn remove_cloud_config() -> Result<(), std::io::Error> {
        let config_paths = SaveData::get_cloud_config_paths();
        let config_file_path = config_paths.1;

        fs::remove_file(config_file_path)?;
        Ok(())
    }

    /// Loads tasks from the application's data file into the SaveData instance.
    ///
    /// If the data directory does not exist it will be created. If the data file does not exist,
    /// no changes are made to the current tasks and the method returns `Ok(())`.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or a `serde_json::Error` if the data file exists but cannot be deserialized.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut sd = SaveData::new();
    /// // If no data file is present this will succeed and leave tasks empty.
    /// sd.load_tasks().unwrap();
    /// assert!(sd.get_tasks().is_empty());
    /// ```
    pub fn load_tasks(&mut self) -> Result<(), serde_json::Error> {
        let data_paths = SaveData::get_data_paths();
        let app_dirs = data_paths.0;
        let data_file_path = data_paths.1;

        fs::create_dir_all(&app_dirs.data_dir).unwrap();

        if !data_file_path.exists() {
            return Ok(());
        }

        let file = File::open(data_file_path).unwrap();

        let result: Vec<Task> = from_reader(file)?;
        self.tasks = result;

        return Ok(());
    }

    pub fn save_tasks(&self) -> Result<(), serde_json::Error> {
        let app_dirs = AppDirs::new(Some("todo"), true).unwrap();
        let data_file_path = app_dirs.data_dir.join("todos.json");

        fs::create_dir_all(&app_dirs.data_dir).unwrap();

        let file = File::create(data_file_path).unwrap();

        to_writer(file, &self.tasks)?;

        return Ok(());
    }

    pub fn get_tasks(&mut self) -> &mut Vec<Task> {
        return &mut self.tasks;
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task)
    }

    pub fn remove_task(&mut self, task_index: usize) {
        self.tasks.remove(task_index);
    }

    pub fn mark_task_done(&mut self, task_index: usize) -> bool {
        let was_done = self.tasks[task_index].done;
        self.tasks[task_index].done = true;
        return was_done;
    }

    pub fn mark_task_undone(&mut self, task_index: usize) -> bool {
        let was_undone = !self.tasks[task_index].done;
        self.tasks[task_index].done = false;
        return was_undone;
    }

    pub fn clear_tasks(&mut self) {
        self.tasks.clear();
    }

    pub fn clear_done_tasks(&mut self) {
        self.tasks.retain(|t| !t.done);
    }
}
