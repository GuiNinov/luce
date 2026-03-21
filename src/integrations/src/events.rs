use crate::IntegrationType;
use luce_shared::TaskId;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

/// Integration-specific events for external platform coordination
/// Note: Core task events (creation, updates, completion) use the shared LuceEventBus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationEvent {
    /// An external update was received from an integration
    ExternalUpdate {
        task_id: TaskId,
        source: IntegrationType,
        data: serde_json::Value,
    },
    /// A synchronization conflict occurred
    SyncConflict {
        task_id: TaskId,
        conflicts: Vec<SyncConflict>,
    },
    /// External integration webhook received
    WebhookReceived {
        source: IntegrationType,
        payload: serde_json::Value,
    },
    /// Integration sync started
    SyncStarted { integration: IntegrationType },
    /// Integration sync completed
    SyncCompleted {
        integration: IntegrationType,
        stats: IntegrationSyncStats,
    },
    /// Integration sync failed
    SyncFailed {
        integration: IntegrationType,
        error: String,
    },
}

/// Statistics about a synchronization operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationSyncStats {
    pub tasks_synced: usize,
    pub conflicts_resolved: usize,
    pub errors: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub task_id: TaskId,
    pub field: String,
    pub luce_value: serde_json::Value,
    pub external_value: serde_json::Value,
    pub integration_type: IntegrationType,
}

#[derive(Debug, Clone)]
pub struct IntegrationEventBus {
    sender: broadcast::Sender<IntegrationEvent>,
}

impl IntegrationEventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<IntegrationEvent> {
        self.sender.subscribe()
    }

    pub async fn publish(
        &self,
        event: IntegrationEvent,
    ) -> Result<(), broadcast::error::SendError<IntegrationEvent>> {
        match self.sender.send(event) {
            Ok(_) => Ok(()),
            Err(broadcast::error::SendError(event)) => {
                tracing::debug!("No subscribers for event: {:?}", event);
                Ok(())
            }
        }
    }

    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for IntegrationEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_event_bus_creation() {
        let event_bus = IntegrationEventBus::new();
        assert_eq!(event_bus.subscriber_count(), 0);
    }

    #[tokio::test]
    async fn test_event_bus_subscribe() {
        let event_bus = IntegrationEventBus::new();
        let _receiver = event_bus.subscribe();
        assert_eq!(event_bus.subscriber_count(), 1);
    }

    #[tokio::test]
    async fn test_event_bus_publish() {
        let event_bus = IntegrationEventBus::new();
        let mut receiver = event_bus.subscribe();

        let task_id = Uuid::new_v4();
        let event = IntegrationEvent::ExternalUpdate {
            task_id,
            source: IntegrationType::GitHub,
            data: serde_json::json!({"action": "created"}),
        };

        event_bus.publish(event.clone()).await.unwrap();

        let received = receiver.recv().await.unwrap();
        match received {
            IntegrationEvent::ExternalUpdate {
                task_id: received_id,
                source,
                ..
            } => {
                assert_eq!(received_id, task_id);
                assert_eq!(source, IntegrationType::GitHub);
            }
            _ => panic!("Wrong event type received"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let event_bus = IntegrationEventBus::new();
        let mut receiver1 = event_bus.subscribe();
        let mut receiver2 = event_bus.subscribe();

        assert_eq!(event_bus.subscriber_count(), 2);

        let event = IntegrationEvent::SyncStarted {
            integration: IntegrationType::Linear,
        };

        event_bus.publish(event.clone()).await.unwrap();

        let received1 = receiver1.recv().await.unwrap();
        let received2 = receiver2.recv().await.unwrap();

        match (received1, received2) {
            (
                IntegrationEvent::SyncStarted { integration: int1 },
                IntegrationEvent::SyncStarted { integration: int2 },
            ) => {
                assert_eq!(int1, IntegrationType::Linear);
                assert_eq!(int2, IntegrationType::Linear);
            }
            _ => panic!("Wrong event types received"),
        }
    }

    #[test]
    fn test_sync_conflict_serialization() {
        let conflict = SyncConflict {
            task_id: Uuid::new_v4(),
            field: "status".to_string(),
            luce_value: serde_json::json!("Completed"),
            external_value: serde_json::json!("Done"),
            integration_type: IntegrationType::Linear,
        };

        let serialized = serde_json::to_string(&conflict).unwrap();
        let deserialized: SyncConflict = serde_json::from_str(&serialized).unwrap();

        assert_eq!(conflict.field, deserialized.field);
        assert_eq!(conflict.integration_type, deserialized.integration_type);
    }
}
