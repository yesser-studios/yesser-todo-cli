pub mod save_data;

use serde::{Deserialize, Serialize};

pub use save_data::*;
pub use yesser_todo_errors::db_error::DatabaseError;

/// Checks whether a task's name exactly equals a query string.
///
/// # Examples
///
/// ```
/// use yesser_todo_db::{Task, exactly_matches};
/// let task = Task { name: "Buy milk".into(), done: false };
/// assert!(exactly_matches(&task, "Buy milk"));
/// assert!(!exactly_matches(&task, "buy milk"));
/// ```
///
/// # Returns
///
/// `true` if the task's name equals `query_string`, `false` otherwise.
pub fn exactly_matches(task: &Task, query_string: &str) -> bool {
    task.name == *query_string
}

/// Finds the position of the first task whose name exactly matches the given query string.
///
/// # Returns
///
/// The zero-based index of the matching task, or `None` if no match is found.
///
/// # Examples
///
/// ```
/// use yesser_todo_db::{get_index, Task};
/// let tasks = vec![Task { name: "one".into(), done: false }, Task { name: "two".into(), done: true }];
/// assert_eq!(get_index(&tasks, "two"), Some(1));
/// assert_eq!(get_index(&tasks, "three"), None);
/// ```
pub fn get_index(tasks: &[Task], query_string: &str) -> Option<usize> {
    tasks.iter().position(|r| exactly_matches(r, query_string))
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub name: String,
    pub done: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CloudConfig {
    pub host: String,
    pub port: String,
}

impl CloudConfig {
    /// Construct a `CloudConfig` that owns copies of the provided host and port strings.
    ///
    /// The returned `CloudConfig` contains owned `String` values cloned from the provided references.
    ///
    /// # Examples
    ///
    /// ```
    /// # use yesser_todo_db::CloudConfig;
    /// let host = "example.com".to_string();
    /// let port = "8080".to_string();
    /// let cfg = CloudConfig::new(&host, &port);
    /// assert_eq!(cfg.host, "example.com");
    /// assert_eq!(cfg.port, "8080");
    /// ```
    pub fn new(host: &str, port: &str) -> Self {
        CloudConfig {
            host: host.to_string(),
            port: port.to_string(),
        }
    }
}

pub trait SaveData: Send + Sync {
    fn get_tasks(&mut self) -> &mut Vec<Task>;
    fn add_task(&mut self, task: Task);
    fn remove_task(&mut self, task_index: usize);
    fn mark_task_done(&mut self, task_index: usize) -> bool;
    fn mark_task_undone(&mut self, task_index: usize) -> bool;
    fn clear_tasks(&mut self);
    fn clear_done_tasks(&mut self);
    fn load_tasks(&mut self) -> Result<(), DatabaseError>;
    fn save_tasks(&self) -> Result<(), DatabaseError>;
    fn get_cloud_config(&self) -> Result<Option<(String, String)>, DatabaseError>;
    fn save_cloud_config(&self, host: &str, port: &str) -> Result<(), DatabaseError>;
    fn remove_cloud_config(&self) -> Result<(), DatabaseError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_equality() {
        let task1 = Task {
            name: "test".to_string(),
            done: false,
        };
        let task2 = Task {
            name: "test".to_string(),
            done: false,
        };
        let task3 = Task {
            name: "test".to_string(),
            done: true,
        };
        assert_eq!(task1, task2);
        assert_ne!(task1, task3);
    }

    #[test]
    fn test_task_clone() {
        let task = Task {
            name: "original".to_string(),
            done: false,
        };
        let cloned = task.clone();
        assert_eq!(task, cloned);
    }

    #[test]
    fn test_cloud_config_new() {
        let config = CloudConfig::new("localhost", "8080");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, "8080");
    }

    #[test]
    fn test_cloud_config_clone() {
        let config = CloudConfig::new("example.com", "443");
        let cloned = config.clone();
        assert_eq!(config.host, cloned.host);
        assert_eq!(config.port, cloned.port);
    }

    #[test]
    fn test_exactly_matches_true() {
        let task = Task {
            name: "Buy milk".to_string(),
            done: false,
        };
        assert!(exactly_matches(&task, "Buy milk"));
    }

    #[test]
    fn test_exactly_matches_false_case() {
        let task = Task {
            name: "Buy milk".to_string(),
            done: false,
        };
        assert!(!exactly_matches(&task, "buy milk"));
    }

    #[test]
    fn test_exactly_matches_false_different() {
        let task = Task {
            name: "Buy milk".to_string(),
            done: false,
        };
        assert!(!exactly_matches(&task, "Buy bread"));
    }

    #[test]
    fn test_get_index_found() {
        let tasks = vec![
            Task {
                name: "one".to_string(),
                done: false,
            },
            Task {
                name: "two".to_string(),
                done: true,
            },
            Task {
                name: "three".to_string(),
                done: false,
            },
        ];
        assert_eq!(get_index(&tasks, "two"), Some(1));
    }

    #[test]
    fn test_get_index_not_found() {
        let tasks = vec![Task {
            name: "one".to_string(),
            done: false,
        }];
        assert_eq!(get_index(&tasks, "two"), None);
    }

    #[test]
    fn test_get_index_empty_list() {
        let tasks: Vec<Task> = vec![];
        assert_eq!(get_index(&tasks, "any"), None);
    }

    #[test]
    fn test_get_index_first_match() {
        let tasks = vec![
            Task {
                name: "duplicate".to_string(),
                done: false,
            },
            Task {
                name: "duplicate".to_string(),
                done: true,
            },
        ];
        assert_eq!(get_index(&tasks, "duplicate"), Some(0));
    }

    #[test]
    fn test_task_serialization() {
        let task = Task {
            name: "test".to_string(),
            done: true,
        };
        let json = serde_json::to_string(&task).unwrap();
        assert!(json.contains("\"name\""));
        assert!(json.contains("\"done\""));
        assert!(json.contains("test"));
    }

    #[test]
    fn test_task_deserialization() {
        let json = r#"{"name":"test task","done":false}"#;
        let task: Task = serde_json::from_str(json).unwrap();
        assert_eq!(task.name, "test task");
        assert!(!task.done);
    }

    #[test]
    fn test_cloud_config_serialization() {
        let config = CloudConfig::new("localhost", "8080");
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("localhost"));
        assert!(json.contains("8080"));
    }

    #[test]
    fn test_cloud_config_deserialization() {
        let json = r#"{"host":"example.com","port":"443"}"#;
        let config: CloudConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.host, "example.com");
        assert_eq!(config.port, "443");
    }
}
