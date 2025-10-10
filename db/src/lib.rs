use std::{fs::{self, File}, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{to_writer, from_reader};
use platform_dirs::AppDirs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub name: String,
    pub done: bool
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CloudConfig {
    pub host: String,
    pub port: String
}

impl CloudConfig {
    pub fn new(host: &String, port: &String) -> Self {
        CloudConfig{host: host.clone(), port: port.clone()}
    }
}

pub struct SaveData {
    tasks: Vec<Task>
}

pub fn exactly_matches(task: &Task, query_string: &String) -> bool {
    return task.name == *query_string;
}

pub fn get_index(tasks: &Vec<Task>, query_string: &String) -> Option<usize> {
    return tasks.iter().position(|r| exactly_matches(r, query_string))
}

impl SaveData {
    pub fn new() -> SaveData {
        return SaveData {tasks: Vec::new()}
    }

    pub(crate) fn get_data_paths() -> (AppDirs, PathBuf) {
        let app_dirs = AppDirs::new(Some("todo"), true).unwrap();
        let data_file_path = app_dirs.data_dir.join("todos.json");
        return (app_dirs, data_file_path);
    }

    pub(crate) fn get_cloud_config_paths() -> (AppDirs, PathBuf) {
        let app_dirs = AppDirs::new(Some("todo"), true).unwrap();
        let config_file_path = app_dirs.config_dir.join("cloud.json");
        return (app_dirs, config_file_path);
    }

    pub fn get_cloud_config() -> Result<Option<(String, String)>, serde_json::Error> {
        let config_paths = SaveData::get_cloud_config_paths();
        let app_dirs = config_paths.0;
        let config_file_path = config_paths.1;

        fs::create_dir_all(&app_dirs.config_dir).unwrap();

        if !config_file_path.exists() {return Ok(None)}

        let file = File::open(config_file_path).unwrap();
        let result: CloudConfig = from_reader(file)?;

        Ok(Some((result.host.clone(), result.port.clone())))
    }

    pub fn save_cloud_config(host: &String, port: &String) -> Result<(), serde_json::Error> {
        let config_paths = SaveData::get_cloud_config_paths();
        let app_dirs = config_paths.0;
        let config_file_path = config_paths.1;

        fs::create_dir_all(&app_dirs.config_dir).unwrap();
        let file = File::create(config_file_path).unwrap();
        to_writer(file, &CloudConfig::new(host, port))?;

        Ok(())
    }
    
    pub fn remove_cloud_config() -> Result<(), std::io::Error> {
        let config_paths = SaveData::get_cloud_config_paths();
        let config_file_path = config_paths.1;
        
        fs::remove_file(config_file_path)?;
        Ok(())
    }

    pub fn load_tasks(&mut self) -> Result<(), serde_json::Error> {
        let data_paths = SaveData::get_data_paths();
        let app_dirs = data_paths.0;
        let data_file_path = data_paths.1;

        fs::create_dir_all(&app_dirs.data_dir).unwrap();

        if !data_file_path.exists() {return Ok(())}

        let file = File::open(data_file_path).unwrap();

        let result: Vec<Task> = from_reader(file)?;
        self.tasks = result;

        return Ok(())
    }

    pub fn save_tasks(&self) -> Result<(), serde_json::Error> {
        let app_dirs = AppDirs::new(Some("todo"), true).unwrap();
        let data_file_path = app_dirs.data_dir.join("todos.json");

        fs::create_dir_all(&app_dirs.data_dir).unwrap();

        let file = File::create(data_file_path).unwrap();

        to_writer(file, &self.tasks)?;

        return Ok(())
    }

    pub fn get_tasks(&self) -> &Vec<Task> {
        return &self.tasks;
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
        return was_done
    }

    pub fn mark_task_undone(&mut self, task_index: usize) -> bool {
        let was_undone = !self.tasks[task_index].done;
        self.tasks[task_index].done = false;
        return was_undone
    }

    pub fn clear_tasks(&mut self) {
        self.tasks.clear();
    }

    pub fn clear_done_tasks(&mut self) {
        self.tasks.retain(|t| !t.done);
    }
}