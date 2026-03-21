use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub type TaskId = Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Ready,
    InProgress,
    Completed,
    Failed,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum TaskPriority {
    Low = 1,
    #[default]
    Normal = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub dependencies: HashSet<TaskId>,
    pub dependents: HashSet<TaskId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub assigned_session: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl Task {
    pub fn new(title: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description: None,
            status: TaskStatus::Pending,
            priority: TaskPriority::default(),
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            assigned_session: None,
            metadata: HashMap::new(),
        }
    }

    pub fn new_with_id(id: TaskId, title: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            title,
            description: None,
            status: TaskStatus::Pending,
            priority: TaskPriority::default(),
            dependencies: HashSet::new(),
            dependents: HashSet::new(),
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            assigned_session: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self.updated_at = Utc::now();
        self
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self.updated_at = Utc::now();
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
        self
    }

    pub fn add_dependency(&mut self, dependency_id: TaskId) {
        self.dependencies.insert(dependency_id);
        self.updated_at = Utc::now();
    }

    pub fn remove_dependency(&mut self, dependency_id: TaskId) {
        self.dependencies.remove(&dependency_id);
        self.updated_at = Utc::now();
    }

    pub fn add_dependent(&mut self, dependent_id: TaskId) {
        self.dependents.insert(dependent_id);
        self.updated_at = Utc::now();
    }

    pub fn remove_dependent(&mut self, dependent_id: TaskId) {
        self.dependents.remove(&dependent_id);
        self.updated_at = Utc::now();
    }

    pub fn set_status(&mut self, status: TaskStatus) {
        let now = Utc::now();

        match (&self.status, &status) {
            (_, TaskStatus::InProgress) if self.started_at.is_none() => {
                self.started_at = Some(now);
            }
            (_, TaskStatus::Completed) | (_, TaskStatus::Failed) if self.completed_at.is_none() => {
                self.completed_at = Some(now);
            }
            _ => {}
        }

        self.status = status;
        self.updated_at = now;
    }

    pub fn assign_to_session(&mut self, session_id: String) {
        self.assigned_session = Some(session_id);
        self.updated_at = Utc::now();
    }

    pub fn unassign_session(&mut self) {
        self.assigned_session = None;
        self.updated_at = Utc::now();
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.status, TaskStatus::Ready)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.status, TaskStatus::Pending)
    }

    pub fn is_in_progress(&self) -> bool {
        matches!(self.status, TaskStatus::InProgress)
    }

    pub fn is_completed(&self) -> bool {
        matches!(self.status, TaskStatus::Completed)
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.status, TaskStatus::Failed)
    }

    pub fn is_blocked(&self) -> bool {
        matches!(self.status, TaskStatus::Blocked)
    }

    pub fn is_terminal(&self) -> bool {
        self.is_completed() || self.is_failed()
    }

    pub fn is_assigned(&self) -> bool {
        self.assigned_session.is_some()
    }

    pub fn has_dependencies(&self) -> bool {
        !self.dependencies.is_empty()
    }

    pub fn has_dependents(&self) -> bool {
        !self.dependents.is_empty()
    }

    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }

    pub fn remove_metadata(&mut self, key: &str) -> Option<String> {
        let result = self.metadata.remove(key);
        if result.is_some() {
            self.updated_at = Utc::now();
        }
        result
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn clear_metadata(&mut self) {
        if !self.metadata.is_empty() {
            self.metadata.clear();
            self.updated_at = Utc::now();
        }
    }

    pub fn duration_since_created(&self) -> chrono::Duration {
        Utc::now().signed_duration_since(self.created_at)
    }

    pub fn duration_since_started(&self) -> Option<chrono::Duration> {
        self.started_at.map(|started| {
            self.completed_at
                .unwrap_or_else(Utc::now)
                .signed_duration_since(started)
        })
    }

    pub fn total_duration(&self) -> Option<chrono::Duration> {
        if let (Some(started), Some(completed)) = (self.started_at, self.completed_at) {
            Some(completed.signed_duration_since(started))
        } else {
            None
        }
    }

    pub fn set_description(&mut self, description: Option<String>) {
        self.description = description;
        self.updated_at = Utc::now();
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
        self.updated_at = Utc::now();
    }

    pub fn set_priority(&mut self, priority: TaskPriority) {
        self.priority = priority;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_task_creation() {
        let task = Task::new("Test task".to_string());
        assert_eq!(task.title, "Test task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert_eq!(task.priority, TaskPriority::Normal);
        assert!(task.dependencies.is_empty());
        assert!(task.dependents.is_empty());
        assert!(task.description.is_none());
        assert!(task.assigned_session.is_none());
        assert!(task.metadata.is_empty());
        assert!(!task.has_dependencies());
        assert!(!task.has_dependents());
        assert!(!task.is_assigned());
    }

    #[test]
    fn test_task_creation_with_id() {
        let id = Uuid::new_v4();
        let task = Task::new_with_id(id, "Test task".to_string());
        assert_eq!(task.id, id);
        assert_eq!(task.title, "Test task");
    }

    #[test]
    fn test_task_builder_methods() {
        let task = Task::new("Test task".to_string())
            .with_description("A test task".to_string())
            .with_priority(TaskPriority::High)
            .with_metadata("category".to_string(), "testing".to_string());

        assert_eq!(task.description, Some("A test task".to_string()));
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.get_metadata("category"), Some(&"testing".to_string()));
    }

    #[test]
    fn test_task_status_transitions() {
        let mut task = Task::new("Test task".to_string());

        assert!(task.is_pending());
        assert!(!task.is_ready());
        assert!(!task.is_in_progress());
        assert!(!task.is_completed());
        assert!(!task.is_failed());
        assert!(!task.is_blocked());
        assert!(!task.is_terminal());

        task.set_status(TaskStatus::Ready);
        assert!(!task.is_pending());
        assert!(task.is_ready());
        assert!(!task.is_terminal());

        task.set_status(TaskStatus::InProgress);
        assert!(task.is_in_progress());
        assert!(task.started_at.is_some());
        assert!(!task.is_terminal());

        task.set_status(TaskStatus::Completed);
        assert!(task.is_completed());
        assert!(task.completed_at.is_some());
        assert!(task.is_terminal());

        let mut task2 = Task::new("Test task 2".to_string());
        task2.set_status(TaskStatus::Failed);
        assert!(task2.is_failed());
        assert!(task2.is_terminal());

        let mut task3 = Task::new("Test task 3".to_string());
        task3.set_status(TaskStatus::Blocked);
        assert!(task3.is_blocked());
        assert!(!task3.is_terminal());
    }

    #[test]
    fn test_task_assignment() {
        let mut task = Task::new("Test task".to_string());

        assert!(!task.is_assigned());

        task.assign_to_session("session-1".to_string());
        assert!(task.is_assigned());
        assert_eq!(task.assigned_session, Some("session-1".to_string()));

        task.unassign_session();
        assert!(!task.is_assigned());
        assert!(task.assigned_session.is_none());
    }

    #[test]
    fn test_task_metadata_management() {
        let mut task = Task::new("Test task".to_string());

        task.add_metadata("key1".to_string(), "value1".to_string());
        task.add_metadata("key2".to_string(), "value2".to_string());

        assert_eq!(task.get_metadata("key1"), Some(&"value1".to_string()));
        assert_eq!(task.get_metadata("key2"), Some(&"value2".to_string()));
        assert_eq!(task.get_metadata("nonexistent"), None);

        let removed = task.remove_metadata("key1");
        assert_eq!(removed, Some("value1".to_string()));
        assert_eq!(task.get_metadata("key1"), None);

        task.clear_metadata();
        assert!(task.metadata.is_empty());
    }

    #[test]
    fn test_task_dependency_management() {
        let mut task = Task::new("Test task".to_string());
        let dep_id = Uuid::new_v4();
        let dependent_id = Uuid::new_v4();

        assert!(!task.has_dependencies());
        assert!(!task.has_dependents());

        task.add_dependency(dep_id);
        assert!(task.has_dependencies());
        assert!(task.dependencies.contains(&dep_id));

        task.add_dependent(dependent_id);
        assert!(task.has_dependents());
        assert!(task.dependents.contains(&dependent_id));

        task.remove_dependency(dep_id);
        assert!(!task.has_dependencies());
        assert!(!task.dependencies.contains(&dep_id));

        task.remove_dependent(dependent_id);
        assert!(!task.has_dependents());
        assert!(!task.dependents.contains(&dependent_id));
    }

    #[test]
    fn test_task_priority_ordering() {
        let low = TaskPriority::Low;
        let normal = TaskPriority::Normal;
        let high = TaskPriority::High;
        let critical = TaskPriority::Critical;

        assert!(low < normal);
        assert!(normal < high);
        assert!(high < critical);

        assert_eq!(TaskPriority::default(), TaskPriority::Normal);
    }

    #[test]
    fn test_task_duration_calculations() {
        let mut task = Task::new("Test task".to_string());

        // Test duration since created
        let duration = task.duration_since_created();
        assert!(duration.num_milliseconds() >= 0);

        // Test duration since started (none when not started)
        assert!(task.duration_since_started().is_none());
        assert!(task.total_duration().is_none());

        // Start the task
        task.set_status(TaskStatus::InProgress);
        thread::sleep(Duration::from_millis(1));

        // Now we should have a duration since started
        let started_duration = task.duration_since_started();
        assert!(started_duration.is_some());
        assert!(started_duration.unwrap().num_milliseconds() >= 0);

        // Complete the task
        task.set_status(TaskStatus::Completed);

        // Now we should have a total duration
        let total = task.total_duration();
        assert!(total.is_some());
        assert!(total.unwrap().num_milliseconds() >= 0);
    }

    #[test]
    fn test_task_setters() {
        let mut task = Task::new("Original title".to_string());

        task.set_title("New title".to_string());
        assert_eq!(task.title, "New title");

        task.set_description(Some("New description".to_string()));
        assert_eq!(task.description, Some("New description".to_string()));

        task.set_description(None);
        assert_eq!(task.description, None);

        task.set_priority(TaskPriority::Critical);
        assert_eq!(task.priority, TaskPriority::Critical);
    }

    #[test]
    fn test_task_serialization() {
        let task = Task::new("Test task".to_string())
            .with_description("A test task".to_string())
            .with_priority(TaskPriority::High);

        let serialized = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&serialized).unwrap();

        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.title, deserialized.title);
        assert_eq!(task.description, deserialized.description);
        assert_eq!(task.priority, deserialized.priority);
        assert_eq!(task.status, deserialized.status);
    }

    #[test]
    fn test_task_status_timestamp_transitions() {
        let mut task = Task::new("Test task".to_string());

        // Multiple status changes to InProgress should only set started_at once
        task.set_status(TaskStatus::InProgress);
        let first_start = task.started_at.unwrap();

        thread::sleep(Duration::from_millis(1));
        task.set_status(TaskStatus::InProgress);
        assert_eq!(task.started_at.unwrap(), first_start);

        // Multiple completions should only set completed_at once
        task.set_status(TaskStatus::Completed);
        let first_completion = task.completed_at.unwrap();

        thread::sleep(Duration::from_millis(1));
        task.set_status(TaskStatus::Completed);
        assert_eq!(task.completed_at.unwrap(), first_completion);
    }

    #[test]
    fn test_task_updated_at_changes() {
        let mut task = Task::new("Test task".to_string());
        let initial_updated = task.updated_at;

        thread::sleep(Duration::from_millis(1));
        task.set_title("New title".to_string());
        assert!(task.updated_at > initial_updated);

        let prev_updated = task.updated_at;
        thread::sleep(Duration::from_millis(1));
        task.add_metadata("key".to_string(), "value".to_string());
        assert!(task.updated_at > prev_updated);
    }
}
