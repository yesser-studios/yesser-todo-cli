use axum::{debug_handler, Json};
use axum::http::StatusCode;
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

#[debug_handler]
pub async fn done_task(Json(index): Json<usize>) -> (StatusCode, Json<Task>) {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    if save_data.get_tasks().len() >= index {
        return (StatusCode::NOT_FOUND, Json(Task{name: "Could not find specified index".to_string(), done: false}));
    }
    println!("Marking task with index {} as done: {}", index, save_data.get_tasks()[index].name);
    save_data.mark_task_done(index);
    save_data.save_tasks().unwrap();
    (StatusCode::OK, Json(save_data.get_tasks()[index].clone()))
}

#[debug_handler]
pub async fn undone_task(Json(index): Json<usize>) -> (StatusCode, Json<Task>) {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    if save_data.get_tasks().len() >= index {
        return (StatusCode::NOT_FOUND, Json(Task{name: "Could not find specified index".to_string(), done: false}));
    }
    println!("Marking task with index {} as done: {}", index, save_data.get_tasks()[index].name);
    save_data.mark_task_undone(index);
    save_data.save_tasks().unwrap();
    (StatusCode::OK, Json(save_data.get_tasks()[index].clone()))
}

#[debug_handler]
pub async fn clear_tasks() {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    println!("Clearing tasks");
    save_data.clear_tasks();
    save_data.save_tasks().unwrap();
}

#[debug_handler]
pub async fn clear_done_tasks() {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    println!("Clearing done tasks");
    save_data.clear_done_tasks();
    save_data.save_tasks().unwrap();
}

#[debug_handler]
pub async fn get_index(Json(name): Json<String>) -> (StatusCode, Json<usize>) {
    let mut save_data = SaveData::new();
    let _ = save_data.load_tasks();
    let result = yesser_todo_db::get_index(save_data.get_tasks(), &name);
    save_data.save_tasks().unwrap();
    match result {
        None => {(StatusCode::NOT_FOUND, Json(0))}
        Some(result) => {(StatusCode::OK, Json(result))}
    }
}