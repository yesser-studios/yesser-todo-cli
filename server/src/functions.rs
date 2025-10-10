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
pub async fn add_task(Json(name): Json<String>) -> Json<Task> {
    println!("Adding task {}", name);
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    let task = Task{name, done: false};
    save_data.add_task(task.clone());
    save_data.save_tasks().unwrap();
    Json(task)
}

#[debug_handler]
pub async fn remove_task(Json(index): Json<usize>) {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    println!("Removing task with index {}: {}", index, save_data.get_tasks()[index].name);
    save_data.remove_task(index);
    save_data.save_tasks().unwrap();
}