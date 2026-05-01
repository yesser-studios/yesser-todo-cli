use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{Json, debug_handler};
use tokio::sync::Mutex;
use yesser_todo_db::{SaveData, Task};

use yesser_todo_errors::server_error::ServerError;

use crate::db_error_wrap::DatabaseErrorWrapper;
use crate::queries::{IndexQuery, NameQuery};

/// Returns the current list of stored tasks as JSON.
#[debug_handler]
pub async fn get_tasks(State(save_data): State<Arc<Mutex<SaveData>>>) -> Result<(StatusCode, Json<Vec<Task>>), ServerError> {
    let tasks = {
        let mut save_data = save_data.lock().await;
        save_data.get_tasks().clone()
    };
    Ok((StatusCode::OK, Json(tasks)))
}

/// Creates and persists a new task with the given name.
///
/// The task is created with `done = false`, saved to persistent storage, and returned as `Json<Task>`.
#[debug_handler]
pub async fn add_task(State(save_data): State<Arc<Mutex<SaveData>>>, Json(name): Json<String>) -> Result<(StatusCode, Json<Task>), ServerError> {
    println!("Adding task {}", name);
    let mut save_data = save_data.lock().await;
    if save_data.get_tasks().iter().any(|t| t.name == name) {
        return Err(ServerError::Conflict(name.into()));
    }

    let task = Task { name, done: false };
    save_data.add_task(task.clone());
    save_data.save_tasks().map_err(DatabaseErrorWrapper::from)?;
    Ok((StatusCode::OK, Json(task)))
}

/// Remove the task at the given zero-based index.
///
/// Attempts to delete the task identified by `index` from persistent storage.
/// Returns `StatusCode::NOT_FOUND` if `index` is outside the current task list.
///
/// # Returns
///
/// `StatusCode::OK` if the task was removed, `StatusCode::NOT_FOUND` if the index is out of bounds.
#[debug_handler]
pub async fn remove_task(State(save_data): State<Arc<Mutex<SaveData>>>, Query(query): Query<IndexQuery>) -> Result<StatusCode, ServerError> {
    let index = query.index;
    let mut save_data = save_data.lock().await;

    save_data.get_tasks().get(index).ok_or(ServerError::NotFound(index.into()))?;

    println!("Removing task with index {}: {}", index, save_data.get_tasks()[index].name);
    save_data.remove_task(index);

    save_data.save_tasks().map_err(DatabaseErrorWrapper::from)?;
    Ok(StatusCode::OK)
}

/// Mark the task at the given index as done.
///
/// Returns `StatusCode::OK` and the updated `Task` when the index is valid. If the index is out of bounds,
/// returns `StatusCode::NOT_FOUND` and a `Task` with `name` set to `"Could not find specified index"` and
/// `done` set to `false`.
#[debug_handler]
pub async fn done_task(State(save_data): State<Arc<Mutex<SaveData>>>, Json(index): Json<usize>) -> Result<(StatusCode, Json<Task>), ServerError> {
    let mut save_data = save_data.lock().await;

    save_data.get_tasks().get(index).ok_or(ServerError::NotFound(index.into()))?;

    println!("Marking task with index {} as done: {}", index, save_data.get_tasks()[index].name);
    save_data.mark_task_done(index);
    save_data.save_tasks().map_err(DatabaseErrorWrapper::from)?;
    Ok((StatusCode::OK, Json(save_data.get_tasks()[index].clone())))
}

/// Mark the task at the given index as not completed and persist the change.
///
/// On success returns `StatusCode::OK` and the updated `Task`. If the index is out of bounds
/// returns `StatusCode::NOT_FOUND` and a `Task` with `name` set to `"Could not find specified index"`
/// and `done` set to `false`.
#[debug_handler]
pub async fn undone_task(State(save_data): State<Arc<Mutex<SaveData>>>, Json(index): Json<usize>) -> Result<(StatusCode, Json<Task>), ServerError> {
    let mut save_data = save_data.lock().await;

    save_data.get_tasks().get(index).ok_or(ServerError::NotFound(index.into()))?;

    println!("Marking task with index {} as undone: {}", index, save_data.get_tasks()[index].name);
    save_data.mark_task_undone(index);
    save_data.save_tasks().map_err(DatabaseErrorWrapper::from)?;
    Ok((StatusCode::OK, Json(save_data.get_tasks()[index].clone())))
}

/// Clears all tasks from persistent storage and persists the empty task list.
///
/// This loads the current tasks, removes every task, and saves the resulting empty list.
#[debug_handler]
pub async fn clear_tasks(State(save_data): State<Arc<Mutex<SaveData>>>) -> Result<StatusCode, ServerError> {
    let mut save_data = save_data.lock().await;

    println!("Clearing tasks");
    save_data.clear_tasks();
    save_data.save_tasks().map_err(DatabaseErrorWrapper::from)?;
    Ok(StatusCode::OK)
}

/// Remove all tasks that are marked as done and persist the updated task list.
///
/// This loads the current tasks, removes any entries where `done == true`,
/// and saves the resulting task list back to storage.
#[debug_handler]
pub async fn clear_done_tasks(State(save_data): State<Arc<Mutex<SaveData>>>) -> Result<StatusCode, ServerError> {
    let mut save_data = save_data.lock().await;

    println!("Clearing done tasks");
    save_data.clear_done_tasks();
    save_data.save_tasks().map_err(DatabaseErrorWrapper::from)?;
    Ok(StatusCode::OK)
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
#[debug_handler]
pub async fn get_index(State(save_data): State<Arc<Mutex<SaveData>>>, Query(params): Query<NameQuery>) -> Result<(StatusCode, Json<usize>), ServerError> {
    let name = params.name;
    let mut save_data = save_data.lock().await;
    match yesser_todo_db::get_index(save_data.get_tasks(), &name) {
        None => Err(ServerError::NotFound(name.into())),
        Some(result) => Ok((StatusCode::OK, Json(result))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_tasks_empty() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        let (status, Json(tasks)) = get_tasks(State(save_data.clone())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_add_task() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        let task_name = "Test task".to_string();
        let (status, Json(task)) = add_task(State(save_data.clone()), Json(task_name.clone())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(task.name, task_name);
        assert!(!task.done);
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_add_multiple_tasks() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 1".to_string())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 2".to_string())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 3".to_string())).await.unwrap();
        let (status, Json(tasks)) = get_tasks(State(save_data.clone())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(tasks.len(), 3);
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_remove_task_success() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 1".to_string())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 2".to_string())).await.unwrap();
        let status = remove_task(State(save_data.clone()), Query(0.into())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        let (status, Json(tasks)) = get_tasks(State(save_data.clone())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "Task 2");
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_remove_task_not_found() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        let err = remove_task(State(save_data.clone()), Query(0.into())).await.unwrap_err();
        assert!(matches!(err, ServerError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_remove_task_out_of_bounds() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 1".to_string())).await.unwrap();
        let err = remove_task(State(save_data.clone()), Query(99.into())).await.unwrap_err();
        assert!(matches!(err, ServerError::NotFound(_)));
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_done_task_success() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 1".to_string())).await.unwrap();
        let (status, Json(task)) = done_task(State(save_data.clone()), Json(0)).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(task.done);
        assert_eq!(task.name, "Task 1");
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_done_task_not_found() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        let err = done_task(State(save_data.clone()), Json(0)).await.unwrap_err();
        assert!(matches!(err, ServerError::NotFound(_)))
    }

    #[tokio::test]
    async fn test_done_task_out_of_bounds() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 1".to_string())).await.unwrap();
        let err = done_task(State(save_data.clone()), Json(5)).await.unwrap_err();
        assert!(matches!(err, ServerError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_undone_task_success() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 1".to_string())).await.unwrap();
        _ = done_task(State(save_data.clone()), Json(0)).await.unwrap();
        let (status, Json(task)) = undone_task(State(save_data.clone()), Json(0)).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(!task.done);
        assert_eq!(task.name, "Task 1");
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_undone_task_not_found() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        let err = undone_task(State(save_data.clone()), Json(0)).await.unwrap_err();
        assert!(matches!(err, ServerError::NotFound(_)));
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_undone_task_out_of_bounds() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 1".to_string())).await.unwrap();
        let err = undone_task(State(save_data.clone()), Json(10)).await.unwrap_err();
        assert!(matches!(err, ServerError::NotFound(_)));
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_clear_tasks() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        _ = add_task(State(save_data.clone()), Json("Task 1".to_string())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 2".to_string())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 3".to_string())).await.unwrap();
        clear_tasks(State(save_data.clone())).await.unwrap();
        let (status, Json(tasks)) = get_tasks(State(save_data.clone())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_clear_done_tasks() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 1".to_string())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 2".to_string())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 3".to_string())).await.unwrap();
        _ = done_task(State(save_data.clone()), Json(0)).await.unwrap();
        _ = done_task(State(save_data.clone()), Json(2)).await.unwrap();
        _ = clear_done_tasks(State(save_data.clone())).await.unwrap();
        let (status, Json(tasks)) = get_tasks(State(save_data.clone())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "Task 2");
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_clear_done_tasks_none_done() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 1".to_string())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 2".to_string())).await.unwrap();
        _ = clear_done_tasks(State(save_data.clone())).await.unwrap();
        let (status, Json(tasks)) = get_tasks(State(save_data.clone())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(tasks.len(), 2);
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_clear_done_tasks_all_done() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 1".to_string())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 2".to_string())).await.unwrap();
        _ = done_task(State(save_data.clone()), Json(0)).await.unwrap();
        _ = done_task(State(save_data.clone()), Json(1)).await.unwrap();
        clear_done_tasks(State(save_data.clone())).await.unwrap();
        let (status, Json(tasks)) = get_tasks(State(save_data.clone())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_get_index_found() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 1".to_string())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 2".to_string())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 3".to_string())).await.unwrap();
        let (status, Json(index)) = get_index(State(save_data.clone()), Query("Task 2".to_string().into())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(index, 1);
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_index_not_found() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Task 1".to_string())).await.unwrap();
        let err = get_index(State(save_data.clone()), Query("Nonexistent".to_string().into())).await.unwrap_err();
        assert!(matches!(err, ServerError::NotFound(_)));
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_get_index_empty_list() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        let err = get_index(State(save_data.clone()), Query("Any task".to_string().into())).await.unwrap_err();
        assert!(matches!(err, ServerError::NotFound(_)))
    }

    #[tokio::test]
    async fn test_done_undone_cycle() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Cycle task".to_string())).await.unwrap();
        let (status, Json(task)) = done_task(State(save_data.clone()), Json(0)).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(task.done);

        let (status, Json(task)) = undone_task(State(save_data.clone()), Json(0)).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(!task.done);
        let (status, Json(task)) = done_task(State(save_data.clone()), Json(0)).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(task.done);
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_workflow_add_done_clear() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Workflow task 1".to_string())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Workflow task 2".to_string())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Workflow task 3".to_string())).await.unwrap();
        _ = done_task(State(save_data.clone()), Json(1)).await;

        let (status, Json(tasks)) = get_tasks(State(save_data.clone())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(tasks.len(), 3);
        assert!(!tasks[0].done);
        assert!(tasks[1].done);
        assert!(!tasks[2].done);
        _ = clear_done_tasks(State(save_data.clone())).await.unwrap();

        let (status, Json(tasks)) = get_tasks(State(save_data.clone())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(tasks.len(), 2);
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_add_task_with_special_characters() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        let special_name = "Task with spaces & symbols! @#$%ᕚ( Ŧคภςץ )ᕘ".to_string();

        let (status, Json(task)) = add_task(State(save_data.clone()), Json(special_name.clone())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(task.name, special_name);

        let (status, Json(index)) = get_index(State(save_data.clone()), Query(special_name.into())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(status, StatusCode::OK);
        assert_eq!(index, 0);
        clear_tasks(State(save_data.clone())).await.unwrap();
    }

    #[tokio::test]
    async fn test_persistence_across_operations() {
        let mut save_data = SaveData::new();
        save_data.load_tasks().unwrap();
        let save_data = Arc::new(Mutex::new(save_data));

        clear_tasks(State(save_data.clone())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Persist 1".to_string())).await.unwrap();
        _ = add_task(State(save_data.clone()), Json("Persist 2".to_string())).await.unwrap();
        let (status, Json(tasks_before)) = get_tasks(State(save_data.clone())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(tasks_before.len(), 2);
        let (status, Json(tasks_after)) = get_tasks(State(save_data.clone())).await.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(tasks_after.len(), 2);
        assert_eq!(tasks_before[0].name, tasks_after[0].name);
        assert_eq!(tasks_before[1].name, tasks_after[1].name);
        clear_tasks(State(save_data.clone())).await.unwrap();
    }
}
