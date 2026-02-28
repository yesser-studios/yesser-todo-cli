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

/// Creates and persists a new task with the given name.
///
/// The task is created with `done = false`, saved to persistent storage, and returned as `Json<Task>`.
///
/// # Examples
///
/// ```
/// use axum::Json;
/// use yesser_todo_db::Task;
///
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

/// Mark the task at the given index as done.
///
/// Returns `StatusCode::OK` and the updated `Task` when the index is valid. If the index is out of bounds,
/// returns `StatusCode::NOT_FOUND` and a `Task` with `name` set to `"Could not find specified index"` and
/// `done` set to `false`.
///
/// # Examples
///
/// ```
/// # use axum::Json;
/// # use http::StatusCode;
/// # use yesser_todo_db::Task;
/// # async fn run_example() {
/// let (status, Json(task)) = crate::done_task(Json(0usize)).await;
/// assert!(status == StatusCode::OK || status == StatusCode::NOT_FOUND);
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

/// Finds the index of a task by name and returns it as a JSON response.
///
/// If a matching task name is found, returns `StatusCode::OK` with the task's index.
/// If no match is found, returns `StatusCode::NOT_FOUND` with `0`.
///
/// # Returns
///
/// `(StatusCode::OK, Json(index))` when a task with the given name is found.
/// `(StatusCode::NOT_FOUND, Json(0))` when no matching task name exists.
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
#[debug_handler]
pub async fn get_index(Json(name): Json<String>) -> (StatusCode, Json<usize>) {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    let result = yesser_todo_db::get_index(save_data.get_tasks(), &name);
    match result {
        None => (StatusCode::NOT_FOUND, Json(0)),
        Some(result) => (StatusCode::OK, Json(result)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_tasks_empty() {
        clear_tasks().await;
        let Json(tasks) = get_tasks().await;
        assert_eq!(tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_add_task() {
        clear_tasks().await;
        let task_name = "Test task".to_string();
        let Json(task) = add_task(Json(task_name.clone())).await;
        assert_eq!(task.name, task_name);
        assert!(!task.done);
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_add_multiple_tasks() {
        clear_tasks().await;
        add_task(Json("Task 1".to_string())).await;
        add_task(Json("Task 2".to_string())).await;
        add_task(Json("Task 3".to_string())).await;
        let Json(tasks) = get_tasks().await;
        assert_eq!(tasks.len(), 3);
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_remove_task_success() {
        clear_tasks().await;
        add_task(Json("Task 1".to_string())).await;
        add_task(Json("Task 2".to_string())).await;
        let status = remove_task(Json(0)).await;
        assert_eq!(status, StatusCode::OK);
        let Json(tasks) = get_tasks().await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "Task 2");
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_remove_task_not_found() {
        clear_tasks().await;
        let status = remove_task(Json(0)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_remove_task_out_of_bounds() {
        clear_tasks().await;
        add_task(Json("Task 1".to_string())).await;
        let status = remove_task(Json(10)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_done_task_success() {
        clear_tasks().await;
        add_task(Json("Task 1".to_string())).await;
        let (status, Json(task)) = done_task(Json(0)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(task.done);
        assert_eq!(task.name, "Task 1");
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_done_task_not_found() {
        clear_tasks().await;
        let (status, Json(task)) = done_task(Json(0)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(task.name, "Could not find specified index");
        assert!(!task.done);
    }

    #[tokio::test]
    async fn test_done_task_out_of_bounds() {
        clear_tasks().await;
        add_task(Json("Task 1".to_string())).await;
        let (status, Json(task)) = done_task(Json(5)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(task.name, "Could not find specified index");
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_undone_task_success() {
        clear_tasks().await;
        add_task(Json("Task 1".to_string())).await;
        done_task(Json(0)).await;
        let (status, Json(task)) = undone_task(Json(0)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(!task.done);
        assert_eq!(task.name, "Task 1");
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_undone_task_not_found() {
        clear_tasks().await;
        let (status, Json(task)) = undone_task(Json(0)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(task.name, "Could not find specified index");
        assert!(!task.done);
    }

    #[tokio::test]
    async fn test_undone_task_out_of_bounds() {
        clear_tasks().await;
        add_task(Json("Task 1".to_string())).await;
        let (status, Json(task)) = undone_task(Json(10)).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_clear_tasks() {
        add_task(Json("Task 1".to_string())).await;
        add_task(Json("Task 2".to_string())).await;
        add_task(Json("Task 3".to_string())).await;
        clear_tasks().await;
        let Json(tasks) = get_tasks().await;
        assert_eq!(tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_clear_done_tasks() {
        clear_tasks().await;
        add_task(Json("Task 1".to_string())).await;
        add_task(Json("Task 2".to_string())).await;
        add_task(Json("Task 3".to_string())).await;
        done_task(Json(0)).await;
        done_task(Json(2)).await;
        clear_done_tasks().await;
        let Json(tasks) = get_tasks().await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "Task 2");
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_clear_done_tasks_none_done() {
        clear_tasks().await;
        add_task(Json("Task 1".to_string())).await;
        add_task(Json("Task 2".to_string())).await;
        clear_done_tasks().await;
        let Json(tasks) = get_tasks().await;
        assert_eq!(tasks.len(), 2);
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_clear_done_tasks_all_done() {
        clear_tasks().await;
        add_task(Json("Task 1".to_string())).await;
        add_task(Json("Task 2".to_string())).await;
        done_task(Json(0)).await;
        done_task(Json(1)).await;
        clear_done_tasks().await;
        let Json(tasks) = get_tasks().await;
        assert_eq!(tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_get_index_found() {
        clear_tasks().await;
        add_task(Json("Task 1".to_string())).await;
        add_task(Json("Task 2".to_string())).await;
        add_task(Json("Task 3".to_string())).await;
        let (status, Json(index)) = get_index(Json("Task 2".to_string())).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(index, 1);
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_get_index_not_found() {
        clear_tasks().await;
        add_task(Json("Task 1".to_string())).await;
        let (status, Json(index)) = get_index(Json("Nonexistent".to_string())).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(index, 0);
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_get_index_empty_list() {
        clear_tasks().await;
        let (status, Json(index)) = get_index(Json("Any task".to_string())).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(index, 0);
    }

    #[tokio::test]
    async fn test_done_undone_cycle() {
        clear_tasks().await;
        add_task(Json("Cycle task".to_string())).await;
        let (status, Json(task)) = done_task(Json(0)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(task.done);
        let (status, Json(task)) = undone_task(Json(0)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(!task.done);
        let (status, Json(task)) = done_task(Json(0)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(task.done);
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_workflow_add_done_clear() {
        clear_tasks().await;
        add_task(Json("Workflow task 1".to_string())).await;
        add_task(Json("Workflow task 2".to_string())).await;
        add_task(Json("Workflow task 3".to_string())).await;
        done_task(Json(1)).await;
        let Json(tasks) = get_tasks().await;
        assert_eq!(tasks.len(), 3);
        assert!(!tasks[0].done);
        assert!(tasks[1].done);
        assert!(!tasks[2].done);
        clear_done_tasks().await;
        let Json(tasks) = get_tasks().await;
        assert_eq!(tasks.len(), 2);
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_add_task_with_special_characters() {
        clear_tasks().await;
        let special_name = "Task with spaces & symbols! @#$%".to_string();
        let Json(task) = add_task(Json(special_name.clone())).await;
        assert_eq!(task.name, special_name);
        let (status, Json(index)) = get_index(Json(special_name)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(index, 0);
        clear_tasks().await;
    }

    #[tokio::test]
    async fn test_persistence_across_operations() {
        clear_tasks().await;
        add_task(Json("Persist 1".to_string())).await;
        add_task(Json("Persist 2".to_string())).await;
        let Json(tasks_before) = get_tasks().await;
        assert_eq!(tasks_before.len(), 2);
        let Json(tasks_after) = get_tasks().await;
        assert_eq!(tasks_after.len(), 2);
        assert_eq!(tasks_before[0].name, tasks_after[0].name);
        assert_eq!(tasks_before[1].name, tasks_after[1].name);
        clear_tasks().await;
    }
}