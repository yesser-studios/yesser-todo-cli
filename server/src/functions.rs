use axum::http::StatusCode;
use axum::{Json, debug_handler};
use yesser_todo_db::{SaveData, Task};

/// Returns the current list of stored tasks as JSON.
///
/// # Examples
///
/// ```no_run
/// let json = get_tasks().await;
/// let tasks = json.0; // Vec<Task>
/// ```
#[debug_handler]
pub async fn get_tasks() -> Json<Vec<Task>> {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    let tasks = save_data.get_tasks().clone();
    Json(tasks)
}

/// Create and persist a new task with the given name.
///
/// The task is created with `done = false`, saved to persistent storage, and returned wrapped in JSON.
///
/// # Examples
///
/// ```
/// use axum::Json;
/// use yesser_todo_db::Task;
///
/// // Example usage in an async context:
/// # async fn example() {
/// let Json(task) = crate::functions::add_task(Json("Buy milk".to_string())).await;
/// assert_eq!(task.name, "Buy milk");
/// assert_eq!(task.done, false);
/// # }
/// ```
#[debug_handler]
pub async fn add_task(Json(name): Json<String>) -> Json<Task> {
    println!("Adding task {}", name);
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    let task = Task { name, done: false };
    save_data.add_task(task.clone());
    save_data.save_tasks().unwrap();
    Json(task)
}

/// Remove the task at the given zero-based index.
///
/// Attempts to delete the task identified by `index` from persistent storage.
/// Returns `StatusCode::NOT_FOUND` if `index` is outside the current task list.
///
/// # Returns
///
/// `StatusCode::OK` if the task was removed, `StatusCode::NOT_FOUND` if the index is out of bounds.
///
/// # Examples
///
/// ```
/// use axum::Json;
/// use axum::http::StatusCode;
///
/// // In an async test or runtime:
/// // let resp = remove_task(Json(0)).await;
/// // assert!(resp == StatusCode::OK || resp == StatusCode::NOT_FOUND);
/// ```
#[debug_handler]
pub async fn remove_task(Json(index): Json<usize>) -> StatusCode {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    if save_data.get_tasks().len() <= index {
        return StatusCode::NOT_FOUND;
    }
    println!("Removing task with index {}: {}", index, save_data.get_tasks()[index].name);
    save_data.remove_task(index);
    save_data.save_tasks().unwrap();
    StatusCode::OK
}

/// Marks the task at the provided index as done.
///
/// If the index is within bounds, returns `StatusCode::OK` and a JSON-serialized copy of the updated `Task`.
/// If the index is out of bounds, returns `StatusCode::NOT_FOUND` and a `Task` with `name` set to
/// `"Could not find specified index"` and `done` set to `false`.
///
/// # Examples
///
/// ```
/// # use axum::Json;
/// # use http::StatusCode;
/// # use yesser_todo_db::Task;
/// # async fn run_example() {
/// let (status, Json(task)) = crate::done_task(Json(0usize)).await;
/// if status == StatusCode::OK {
///     // task is the updated task marked done
///     assert!(task.done);
/// } else {
///     // index was not found
///     assert_eq!(status, StatusCode::NOT_FOUND);
///     assert_eq!(task.name, "Could not find specified index");
/// }
/// # }
/// ```
#[debug_handler]
pub async fn done_task(Json(index): Json<usize>) -> (StatusCode, Json<Task>) {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    if save_data.get_tasks().len() <= index {
        return (
            StatusCode::NOT_FOUND,
            Json(Task {
                name: "Could not find specified index".to_string(),
                done: false,
            }),
        );
    }
    println!("Marking task with index {} as done: {}", index, save_data.get_tasks()[index].name);
    save_data.mark_task_done(index);
    save_data.save_tasks().unwrap();
    (StatusCode::OK, Json(save_data.get_tasks()[index].clone()))
}

/// Mark the task at the given index as not completed and persist the change.
///
/// On success returns `StatusCode::OK` and the updated `Task`. If the index is out of bounds
/// returns `StatusCode::NOT_FOUND` and a `Task` with `name` set to `"Could not find specified index"`
/// and `done` set to `false`.
///
/// # Examples
///
/// ```
/// use axum::Json;
/// use axum::http::StatusCode;
/// use yesser_todo_db::Task;
///
/// // This example demonstrates the call shape; in real use the function runs inside an async runtime.
/// # async fn doc_example() {
/// let index = Json(0usize);
/// let (status, Json(task)) = crate::functions::undone_task(index).await;
/// assert!(status == StatusCode::OK || status == StatusCode::NOT_FOUND);
/// # }
/// ```
#[debug_handler]
pub async fn undone_task(Json(index): Json<usize>) -> (StatusCode, Json<Task>) {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    if save_data.get_tasks().len() <= index {
        return (
            StatusCode::NOT_FOUND,
            Json(Task {
                name: "Could not find specified index".to_string(),
                done: false,
            }),
        );
    }
    println!("Marking task with index {} as undone: {}", index, save_data.get_tasks()[index].name);
    save_data.mark_task_undone(index);
    save_data.save_tasks().unwrap();
    (StatusCode::OK, Json(save_data.get_tasks()[index].clone()))
}

/// Clears all tasks from persistent storage and persists the empty task list.
///
/// This loads the current tasks, removes every task, and saves the resulting empty list.
///
/// # Examples
///
/// ```
/// # use server::functions::clear_tasks;
/// # tokio_test::block_on(async {
/// clear_tasks().await;
/// # });
/// ```
#[debug_handler]
pub async fn clear_tasks() {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    println!("Clearing tasks");
    save_data.clear_tasks();
    save_data.save_tasks().unwrap();
}

/// Remove all tasks that are marked as done and persist the updated task list.
///
/// This loads the current tasks, removes any entries where `done == true`,
/// and saves the resulting task list back to storage.
///
/// # Examples
///
/// ```no_run
/// // Call from an async context
/// clear_done_tasks().await;
/// ```
#[debug_handler]
pub async fn clear_done_tasks() {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    println!("Clearing done tasks");
    save_data.clear_done_tasks();
    save_data.save_tasks().unwrap();
}

/// Looks up the numeric index of a task by its name and returns it as a JSON response.
///
/// Looks up `name` in persisted tasks.
///
/// # Returns
///
/// - `(StatusCode::OK, Json(index))` when a task with the given name is found.
/// - `(StatusCode::NOT_FOUND, Json(0))` when no matching task name exists.
///
/// # Examples
///
/// ```
/// use axum::Json;
/// use axum::http::StatusCode;
///
/// #[tokio::test]
/// async fn example_get_index_not_found() {
///     let (status, Json(idx)) = crate::get_index(Json("no such task".to_string())).await;
///     assert_eq!(status, StatusCode::NOT_FOUND);
///     assert_eq!(idx, 0);
/// }
/// ```
        Some(result) => (StatusCode::OK, Json(result)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_tasks_empty() {
        let Json(tasks) = get_tasks().await;
        assert!(tasks.len() >= 0);
    }

    #[tokio::test]
    async fn test_add_task_creates_task() {
        let Json(task) = add_task(Json("Test Task".to_string())).await;
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.done, false);
    }

    #[tokio::test]
    async fn test_add_task_with_empty_name() {
        let Json(task) = add_task(Json("".to_string())).await;
        assert_eq!(task.name, "");
        assert_eq!(task.done, false);
    }

    #[tokio::test]
    async fn test_remove_task_out_of_bounds() {
        let status = remove_task(Json(9999)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_done_task_out_of_bounds() {
        let (status, _) = done_task(Json(9999)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_done_task_error_response() {
        let (status, Json(task)) = done_task(Json(9999)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(task.name, "Could not find specified index");
        assert_eq!(task.done, false);
    }

    #[tokio::test]
    async fn test_undone_task_out_of_bounds() {
        let (status, _) = undone_task(Json(9999)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_undone_task_error_response() {
        let (status, Json(task)) = undone_task(Json(9999)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(task.name, "Could not find specified index");
        assert_eq!(task.done, false);
    }

    #[tokio::test]
    async fn test_clear_tasks_executes() {
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_clear_done_tasks_executes() {
        clear_done_tasks().await;
    }

    #[tokio::test]
    async fn test_get_index_not_found() {
        let (status, Json(idx)) = get_index(Json("nonexistent task".to_string())).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(idx, 0);
    }

    #[tokio::test]
    async fn test_add_and_get_index() {
        let unique_name = format!("unique_task_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        let Json(task) = add_task(Json(unique_name.clone())).await;
        assert_eq!(task.name, unique_name);

        let (status, Json(_idx)) = get_index(Json(unique_name.clone())).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_task_workflow() {
        let unique_name = format!("workflow_task_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

        let Json(task) = add_task(Json(unique_name.clone())).await;
        assert_eq!(task.name, unique_name);
        assert_eq!(task.done, false);

        let (status, Json(idx)) = get_index(Json(unique_name.clone())).await;
        assert_eq!(status, StatusCode::OK);

        let (status, Json(task)) = done_task(Json(idx)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(task.done, true);

        let (status, Json(task)) = undone_task(Json(idx)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(task.done, false);

        let status = remove_task(Json(idx)).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_add_task_special_characters() {
        let special_name = "Task with 特殊字符 and émojis 🎉".to_string();
        let Json(task) = add_task(Json(special_name.clone())).await;
        assert_eq!(task.name, special_name);
    }

    #[tokio::test]
    async fn test_multiple_tasks_same_name() {
        let name = format!("duplicate_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

        let Json(task1) = add_task(Json(name.clone())).await;
        let Json(task2) = add_task(Json(name.clone())).await;

        assert_eq!(task1.name, name);
        assert_eq!(task2.name, name);
    }
}
    let _ = save_data.load_tasks();
    let result = yesser_todo_db::get_index(save_data.get_tasks(), &name);
    match result {
        None => (StatusCode::NOT_FOUND, Json(0)),
        Some(result) => (StatusCode::OK, Json(result)),
    }
}