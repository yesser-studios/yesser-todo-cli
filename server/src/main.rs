use axum::{debug_handler, routing::{get, post}, Json, Router};
use yesser_todo_db::{SaveData, Task};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/tasks", get(get_tasks))
        .route("/add", post(add_task));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6982").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

#[debug_handler]
async fn get_tasks() -> Json<Vec<Task>> {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    let tasks = save_data.get_tasks().clone();
    Json(tasks)
}

#[debug_handler]
async fn add_task() {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    save_data.add_task(Task{name: String::from("Task"), done: false});
    save_data.save_tasks().unwrap();
}