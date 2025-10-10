mod functions;

use crate::functions::{add_task, clear_done_tasks, clear_tasks, done_task, get_index, get_tasks, remove_task, undone_task};
use axum::routing::delete;
use axum::{routing::{get, post}, Router};

/// Binary entry point that configures HTTP routes and starts the Axum server.
///
/// The function constructs a Router with handlers for task management endpoints
/// and serves it on 0.0.0.0:6982. The server runs until the process exits.
///
/// # Examples
///
/// ```no_run
/// // Run the compiled binary and query the index route:
/// // $ cargo run --bin server
/// // $ curl http://127.0.0.1:6982/index
/// ```
#[tokio::main]
async fn main() {
    let router = Router::new()
        .route("/tasks", get(get_tasks))
        .route("/add", post(add_task))
        .route("/remove", delete(remove_task))
        .route("/done", post(done_task))
        .route("/undone", post(undone_task))
        .route("/clear", delete(clear_tasks))
        .route("/cleardone", delete(clear_done_tasks))
        .route("/index", get(get_index));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6982").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}