use axum::{debug_handler, Json};
use yesser_todo_db::{SaveData, Task};

#[debug_handler]
pub async fn get_tasks() -> Json<Vec<Task>> {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    let tasks = save_data.get_tasks().clone();
    Json(tasks)
}

#[debug_handler]
pub async fn add_task() {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    save_data.add_task(Task{name: String::from("Task"), done: false});
    save_data.save_tasks().unwrap();
}

#[debug_handler]
pub async fn remove_task() {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    save_data.remove_task(1);
    save_data.save_tasks().unwrap();
}