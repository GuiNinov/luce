use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use crate::{TaskId, TaskPriority, TaskStatus};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LuceEvent {
    pub id: String,
    pub event_type: LuceEventType,
    pub timestamp: DateTime<Utc>,
    pub task_id: Option<TaskId>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LuceEventType {
    TaskCreated,
    TaskUpdated,
    TaskCompleted,
    TaskFailed,
    TaskAssigned,
    TaskUnassigned,
    DependencyAdded,
    DependencyRemoved,
    AttachmentAdded,
    AttachmentUpdated,
    AttachmentRemoved,
}

pub type EventHandler = Arc<dyn Fn(&LuceEvent) + Send + Sync>;

#[derive(Clone)]
pub struct LuceEventBus {
    handlers: Arc<Mutex<Vec<EventHandler>>>,
}

impl std::fmt::Debug for LuceEventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LuceEventBus")
            .field(
                "handlers",
                &format!(
                    "Arc<Mutex<Vec<EventHandler>>>({} handlers)",
                    self.handlers.lock().map(|h| h.len()).unwrap_or(0)
                ),
            )
            .finish()
    }
}

impl LuceEventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe(&self, handler: EventHandler) {
        if let Ok(mut handlers) = self.handlers.lock() {
            handlers.push(handler);
        }
    }

    pub fn publish(&self, event: LuceEvent) {
        if let Ok(handlers) = self.handlers.lock() {
            for handler in handlers.iter() {
                handler(&event);
            }
        }
    }

    pub fn create_task_event(
        event_type: LuceEventType,
        task_id: TaskId,
        data: serde_json::Value,
    ) -> LuceEvent {
        LuceEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: Utc::now(),
            task_id: Some(task_id),
            data,
        }
    }

    pub fn create_event(event_type: LuceEventType, data: serde_json::Value) -> LuceEvent {
        LuceEvent {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            timestamp: Utc::now(),
            task_id: None,
            data,
        }
    }
}

impl Default for LuceEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskChanges {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
}

impl TaskChanges {
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            status: None,
            priority: None,
            assignee: None,
        }
    }

    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = Some(status.to_string());
        self
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = Some(priority.to_string());
        self
    }

    pub fn with_assignee(mut self, assignee: String) -> Self {
        self.assignee = Some(assignee);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.description.is_none()
            && self.status.is_none()
            && self.priority.is_none()
            && self.assignee.is_none()
    }
}

impl Default for TaskChanges {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_event_bus_creation() {
        let event_bus = LuceEventBus::new();
        assert!(event_bus.handlers.lock().unwrap().is_empty());
    }

    #[test]
    fn test_event_bus_subscribe_and_publish() {
        let event_bus = LuceEventBus::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let handler = Arc::new(move |_event: &LuceEvent| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        event_bus.subscribe(handler);

        let event = LuceEvent {
            id: "test-event".to_string(),
            event_type: LuceEventType::TaskCreated,
            timestamp: Utc::now(),
            task_id: Some(uuid::Uuid::new_v4()),
            data: serde_json::json!({"test": "data"}),
        };

        event_bus.publish(event);
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_create_task_event() {
        let task_id = uuid::Uuid::new_v4();
        let data = serde_json::json!({"title": "Test Task"});

        let event =
            LuceEventBus::create_task_event(LuceEventType::TaskCreated, task_id, data.clone());

        assert_eq!(event.event_type, LuceEventType::TaskCreated);
        assert_eq!(event.task_id, Some(task_id));
        assert_eq!(event.data, data);
    }

    #[test]
    fn test_create_event() {
        let data = serde_json::json!({"message": "System started"});
        let event = LuceEventBus::create_event(LuceEventType::TaskCreated, data.clone());

        assert_eq!(event.event_type, LuceEventType::TaskCreated);
        assert_eq!(event.task_id, None);
        assert_eq!(event.data, data);
    }

    #[test]
    fn test_task_changes_builder() {
        let changes = TaskChanges::new()
            .with_title("New Title".to_string())
            .with_status(TaskStatus::InProgress)
            .with_priority(TaskPriority::High);

        assert_eq!(changes.title, Some("New Title".to_string()));
        assert_eq!(changes.status, Some("InProgress".to_string()));
        assert_eq!(changes.priority, Some("High".to_string()));
        assert_eq!(changes.description, None);
        assert_eq!(changes.assignee, None);
        assert!(!changes.is_empty());
    }

    #[test]
    fn test_empty_task_changes() {
        let changes = TaskChanges::new();
        assert!(changes.is_empty());

        let changes_with_title = changes.with_title("Test".to_string());
        assert!(!changes_with_title.is_empty());
    }

    #[test]
    fn test_event_serialization() {
        let event = LuceEvent {
            id: "test-event".to_string(),
            event_type: LuceEventType::TaskUpdated,
            timestamp: Utc::now(),
            task_id: Some(uuid::Uuid::new_v4()),
            data: serde_json::json!({"key": "value"}),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: LuceEvent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_event_types_serialization() {
        let event_types = vec![
            LuceEventType::TaskCreated,
            LuceEventType::TaskUpdated,
            LuceEventType::TaskCompleted,
            LuceEventType::TaskFailed,
            LuceEventType::TaskAssigned,
            LuceEventType::TaskUnassigned,
            LuceEventType::DependencyAdded,
            LuceEventType::DependencyRemoved,
            LuceEventType::AttachmentAdded,
            LuceEventType::AttachmentUpdated,
            LuceEventType::AttachmentRemoved,
        ];

        for event_type in event_types {
            let serialized = serde_json::to_string(&event_type).unwrap();
            let deserialized: LuceEventType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(event_type, deserialized);
        }
    }
}
