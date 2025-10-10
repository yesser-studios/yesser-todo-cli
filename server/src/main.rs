mod functions;

use axum::{debug_handler, routing::{get, post}, Json, Router};
use crate::functions::{add_task, get_tasks};

#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/tasks", get(get_tasks))
        .route("/add", post(add_task));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6982").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}