use crate::TaskId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskDependency {
    pub task_id: TaskId,
    pub depends_on_task_id: TaskId,
    pub created_at: DateTime<Utc>,
}

impl TaskDependency {
    pub fn new(task_id: TaskId, depends_on_task_id: TaskId) -> Self {
        Self {
            task_id,
            depends_on_task_id,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_task_dependency_creation() {
        let task_id = Uuid::new_v4();
        let depends_on_id = Uuid::new_v4();

        let dependency = TaskDependency::new(task_id, depends_on_id);

        assert_eq!(dependency.task_id, task_id);
        assert_eq!(dependency.depends_on_task_id, depends_on_id);
        assert!(dependency.created_at <= Utc::now());
    }

    #[test]
    fn test_task_dependency_serialization() {
        let task_id = Uuid::new_v4();
        let depends_on_id = Uuid::new_v4();
        let dependency = TaskDependency::new(task_id, depends_on_id);

        let serialized = serde_json::to_string(&dependency).unwrap();
        let deserialized: TaskDependency = serde_json::from_str(&serialized).unwrap();

        assert_eq!(dependency.task_id, deserialized.task_id);
        assert_eq!(
            dependency.depends_on_task_id,
            deserialized.depends_on_task_id
        );
        assert_eq!(dependency.created_at, deserialized.created_at);
    }
}
