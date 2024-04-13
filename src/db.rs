use json::{stringify,parse};

pub struct Task {
    pub name: String,
    pub done: bool
}

pub(crate) struct SaveData {
    tasks: Vec<Task>
}

impl SaveData {
    pub fn new() -> SaveData {
        return SaveData {tasks: Vec::new()}
    }

    pub fn load_tasks(&mut self) {

    }

    pub fn get_tasks(&self) -> &Vec<Task> {
        return &self.tasks;
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.insert(self.tasks.len(), task)
    }

    pub fn remove_task(&mut self, task_index: usize) {
        self.tasks.remove(task_index);
    }

    pub fn mark_task_done(&mut self, task_index: usize) -> bool {
        let was_done = self.tasks[task_index].done;
        self.tasks[task_index].done = true;
        return was_done
    }

    pub fn mark_task_undone(&mut self, task_index: usize) -> bool {
        let was_undone = !self.tasks[task_index].done;
        self.tasks[task_index].done = false;
        return was_undone
    }
}