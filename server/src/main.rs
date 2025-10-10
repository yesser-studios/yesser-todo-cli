mod functions;

use crate::functions::{add_task, get_tasks, remove_task};
use axum::routing::delete;
use axum::{routing::{get, post}, Router};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/tasks", get(get_tasks))
        .route("/add", post(add_task))
        .route("/remove", delete(remove_task));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6982").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}