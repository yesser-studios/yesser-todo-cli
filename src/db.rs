use std::fs::{self, File};

use serde::{Deserialize, Serialize};
use serde_json::{to_writer, from_reader};
use platform_dirs::AppDirs;

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub done: bool
}

pub(crate) struct SaveData {
    tasks: Vec<Task>
}

pub(crate) fn matches(task: &Task, query_string: &String) -> bool {
    return task.name == *query_string;
}

pub(crate) fn get_index(tasks: &Vec<Task>, query_string: &String) -> Option<usize> {
    return tasks.iter().position(|r| matches(r, query_string))
}

impl SaveData {
    pub fn new() -> SaveData {
        return SaveData {tasks: Vec::new()}
    }

    pub fn load_tasks(&mut self) -> Result<(), serde_json::Error> {
        let app_dirs = AppDirs::new(Some("todo"), true).unwrap();
        let data_file_path = app_dirs.data_dir.join("todos.json");

        println!("Data: {}", data_file_path.as_path().to_str().unwrap());

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
        self.tasks = Vec::new();
    }
}