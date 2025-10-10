use axum::{
    routing::{get, post},
    Router,
};
use yesser_todo_db::{SaveData, Task};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/tasks", get(get_tasks));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6982").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

async fn get_tasks() {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    let tasks = save_data.get_tasks();
    println!("Tasks got");
}
