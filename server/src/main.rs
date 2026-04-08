mod functions;

use std::process::exit;
use std::sync::Arc;

use crate::functions::{add_task, clear_done_tasks, clear_tasks, done_task, get_index, get_tasks, remove_task, undone_task};
use axum::routing::delete;
use axum::{
    Router,
    routing::{get, post},
};
use tokio::sync::Mutex;
use yesser_todo_db::SaveData;

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
    let mut save_data = SaveData::new();
    match save_data.load_tasks() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("An error occurred while loading tasks: {err}");
            exit(1)
        }
    }
    let save_data: Arc<Mutex<SaveData>> = Arc::new(Mutex::new(save_data));

    let router = Router::new()
        .route("/tasks", get(get_tasks))
        .route("/add", post(add_task))
        .route("/remove", delete(remove_task))
        .route("/done", post(done_task))
        .route("/undone", post(undone_task))
        .route("/clear", delete(clear_tasks))
        .route("/cleardone", delete(clear_done_tasks))
        .route("/index", get(get_index))
        .with_state(save_data);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:6982").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
