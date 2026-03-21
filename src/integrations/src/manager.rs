use anyhow::Result;
use luce_shared::{LuceEvent, LuceEventBus, TaskId, TaskPriority};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    ConflictResolutionStrategy, GitHubConfig, GitHubIntegration, IntegrationEvent,
    IntegrationEventBus, IntegrationType, LinearConfig, LinearIntegration, NotificationLevel,
    SlackConfig, SlackIntegration, SyncConflict, SyncEngine, TaskChanges, WebhookHandler,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub github: GitHubConfigWrapper,
    pub linear: LinearConfigWrapper,
    pub slack: SlackConfigWrapper,
    pub sync: SyncConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfigWrapper {
    pub enabled: bool,
    #[serde(flatten)]
    pub config: GitHubConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearConfigWrapper {
    pub enabled: bool,
    #[serde(flatten)]
    pub config: LinearConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfigWrapper {
    pub enabled: bool,
    #[serde(flatten)]
    pub config: SlackConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub conflict_resolution: ConflictResolutionStrategy,
    pub auto_create_external_tasks: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 300, // 5 minutes
            conflict_resolution: ConflictResolutionStrategy::LuceWins,
            auto_create_external_tasks: false,
        }
    }
}

#[derive(Debug)]
pub struct IntegrationManager {
    github: Option<GitHubIntegration>,
    linear: Option<LinearIntegration>,
    slack: Option<SlackIntegration>,
    webhook_handler: Option<WebhookHandler>,
    sync_engine: Option<Arc<RwLock<SyncEngine>>>,
    integration_event_bus: Arc<IntegrationEventBus>,
    luce_event_bus: Arc<LuceEventBus>,
    config: IntegrationConfig,
}

impl IntegrationManager {
    pub async fn new(config: IntegrationConfig) -> Result<Self> {
        let integration_event_bus = Arc::new(IntegrationEventBus::new());
        let luce_event_bus = Arc::new(LuceEventBus::new());

        // Initialize GitHub integration if enabled
        let github = if config.github.enabled {
            Some(GitHubIntegration::new(
                config.github.config.clone(),
                (*integration_event_bus).clone(),
            ))
        } else {
            None
        };

        // Initialize Linear integration if enabled
        let linear = if config.linear.enabled {
            Some(LinearIntegration::new(
                config.linear.config.clone(),
                (*integration_event_bus).clone(),
            ))
        } else {
            None
        };

        // Initialize Slack integration if enabled
        let slack = if config.slack.enabled {
            Some(SlackIntegration::new(
                config.slack.config.clone(),
                (*integration_event_bus).clone(),
            ))
        } else {
            None
        };

        // Initialize webhook handler if any integration is enabled
        let webhook_handler = if github.is_some() || linear.is_some() || slack.is_some() {
            Some(WebhookHandler::new(
                github.clone(),
                linear.clone(),
                slack.clone(),
                (*integration_event_bus).clone(),
            ))
        } else {
            None
        };

        // Initialize sync engine if enabled
        let sync_engine = if config.sync.enabled {
            let engine = SyncEngine::new(
                github.clone(),
                linear.clone(),
                slack.clone(),
                (*integration_event_bus).clone(),
            )
            .with_conflict_resolution_strategy(config.sync.conflict_resolution.clone())
            .with_sync_interval(tokio::time::Duration::from_secs(
                config.sync.interval_seconds,
            ));

            Some(Arc::new(RwLock::new(engine)))
        } else {
            None
        };

        Ok(Self {
            github,
            linear,
            slack,
            webhook_handler,
            sync_engine,
            integration_event_bus,
            luce_event_bus,
            config,
        })
    }

    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting Integration Manager");

        // Start event listener
        self.start_event_listener().await;

        // Start periodic sync if enabled
        if let Some(sync_engine) = &self.sync_engine {
            let engine_clone = Arc::clone(sync_engine);
            tokio::spawn(async move {
                let mut engine = engine_clone.write().await;
                engine.start_periodic_sync().await;
            });
        }

        tracing::info!("Integration Manager started successfully");
        Ok(())
    }

    pub fn get_webhook_handler(&self) -> Option<&WebhookHandler> {
        self.webhook_handler.as_ref()
    }

    pub fn get_webhook_router(&self) -> Option<axum::Router> {
        self.webhook_handler
            .as_ref()
            .map(|handler| handler.clone().router())
    }

    pub async fn handle_task_created(
        &self,
        task_id: TaskId,
        title: &str,
        description: Option<&str>,
    ) -> Result<()> {
        tracing::info!("Handling task creation: {} - {}", task_id, title);

        // Create external tasks if auto-creation is enabled
        if self.config.sync.auto_create_external_tasks {
            self.create_external_tasks(task_id, title, description)
                .await?;
        }

        // Notify Slack about task creation
        if let Some(slack) = &self.slack {
            slack.notify_task_created(task_id, title).await?;
        }

        // Publish core task created event
        let event = LuceEvent::TaskCreated {
            task_id,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            priority: TaskPriority::Normal, // Default priority
        };
        self.luce_event_bus.publish(event).await?;

        tracing::info!("Task creation handled successfully: {}", task_id);
        Ok(())
    }

    pub async fn handle_task_updated(
        &self,
        task_id: TaskId,
        title: &str,
        changes: TaskChanges,
    ) -> Result<()> {
        tracing::info!("Handling task update: {} - {}", task_id, title);

        // Update external systems
        self.update_external_tasks(task_id, &changes).await?;

        // Notify Slack about task update
        if let Some(slack) = &self.slack {
            slack.notify_task_updated(task_id, title, &changes).await?;
        }

        // Publish core task updated event
        let mut metadata = HashMap::new();
        if let Some(priority) = &changes.priority {
            metadata.insert("priority".to_string(), priority.clone());
        }
        if let Some(assignee) = &changes.assignee {
            metadata.insert("assignee".to_string(), assignee.clone());
        }

        let event = LuceEvent::TaskUpdated {
            task_id,
            title: changes.title.clone(),
            description: changes.description.clone(),
            priority: changes.priority.as_ref().and_then(|p| match p.as_str() {
                "Low" => Some(TaskPriority::Low),
                "Normal" => Some(TaskPriority::Normal),
                "High" => Some(TaskPriority::High),
                "Critical" => Some(TaskPriority::Critical),
                _ => None,
            }),
            metadata,
        };
        self.luce_event_bus.publish(event).await?;

        tracing::info!("Task update handled successfully: {}", task_id);
        Ok(())
    }

    pub async fn handle_task_completed(&self, task_id: TaskId, title: &str) -> Result<()> {
        tracing::info!("Handling task completion: {} - {}", task_id, title);

        // Update external systems to mark as completed
        let changes = TaskChanges {
            status: Some("Completed".to_string()),
            title: None,
            description: None,
            priority: None,
            assignee: None,
        };
        self.update_external_tasks(task_id, &changes).await?;

        // Notify Slack about task completion
        if let Some(slack) = &self.slack {
            slack.notify_task_completed(task_id, title).await?;
        }

        // Publish core task completed event
        let event = LuceEvent::TaskCompleted { task_id };
        self.luce_event_bus.publish(event).await?;

        tracing::info!("Task completion handled successfully: {}", task_id);
        Ok(())
    }

    pub async fn handle_task_failed(
        &self,
        task_id: TaskId,
        title: &str,
        reason: &str,
    ) -> Result<()> {
        tracing::info!(
            "Handling task failure: {} - {} ({})",
            task_id,
            title,
            reason
        );

        // Notify Slack about task failure
        if let Some(slack) = &self.slack {
            slack.notify_task_failed(task_id, title, reason).await?;
        }

        // Publish core task failed event
        let event = LuceEvent::TaskFailed {
            task_id,
            reason: reason.to_string(),
        };
        self.luce_event_bus.publish(event).await?;

        tracing::info!("Task failure handled successfully: {}", task_id);
        Ok(())
    }

    pub async fn handle_session_assignment(
        &self,
        task_id: TaskId,
        title: &str,
        session: &str,
        user: &str,
    ) -> Result<()> {
        tracing::info!(
            "Handling session assignment: {} assigned to {} ({})",
            task_id,
            user,
            session
        );

        // Notify Slack about session assignment
        if let Some(slack) = &self.slack {
            slack
                .notify_session_assignment(task_id, title, session, user)
                .await?;
        }

        // Update external systems with assignee
        let changes = TaskChanges {
            status: None,
            title: None,
            description: None,
            priority: None,
            assignee: Some(user.to_string()),
        };
        self.update_external_tasks(task_id, &changes).await?;

        tracing::info!("Session assignment handled successfully: {}", task_id);
        Ok(())
    }

    async fn create_external_tasks(
        &self,
        task_id: TaskId,
        title: &str,
        description: Option<&str>,
    ) -> Result<()> {
        // Create GitHub issue if enabled
        if let Some(github) = &self.github {
            match github.create_issue(task_id, title, description).await {
                Ok(issue) => {
                    tracing::info!(
                        "Created GitHub issue #{} for task {}",
                        issue.number,
                        task_id
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to create GitHub issue for task {}: {}", task_id, e);
                }
            }
        }

        // Create Linear issue if enabled
        if let Some(linear) = &self.linear {
            match linear.create_issue(task_id, title, description).await {
                Ok(issue) => {
                    tracing::info!(
                        "Created Linear issue {} for task {}",
                        issue.identifier,
                        task_id
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to create Linear issue for task {}: {}", task_id, e);
                }
            }
        }

        Ok(())
    }

    async fn update_external_tasks(&self, task_id: TaskId, changes: &TaskChanges) -> Result<()> {
        // Update GitHub issue if enabled
        if let Some(github) = &self.github {
            // TODO: Get GitHub issue number from task metadata
            let issue_number = 1; // Placeholder
            match github.update_issue(issue_number, changes).await {
                Ok(_) => {
                    tracing::debug!("Updated GitHub issue for task {}", task_id);
                }
                Err(e) => {
                    tracing::error!("Failed to update GitHub issue for task {}: {}", task_id, e);
                }
            }
        }

        // Update Linear issue if enabled
        if let Some(linear) = &self.linear {
            // TODO: Get Linear issue ID from task metadata
            let issue_id = "issue_123"; // Placeholder
            match linear.update_issue(issue_id, changes).await {
                Ok(_) => {
                    tracing::debug!("Updated Linear issue for task {}", task_id);
                }
                Err(e) => {
                    tracing::error!("Failed to update Linear issue for task {}: {}", task_id, e);
                }
            }
        }

        Ok(())
    }

    async fn start_event_listener(&self) {
        let mut receiver = self.integration_event_bus.subscribe();

        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                match event {
                    IntegrationEvent::SyncConflict { task_id, conflicts } => {
                        tracing::warn!(
                            "Sync conflict detected for task {}: {} conflicts",
                            task_id,
                            conflicts.len()
                        );
                        // TODO: Handle conflicts based on configuration
                    }
                    IntegrationEvent::ExternalUpdate {
                        task_id,
                        source,
                        data: _,
                    } => {
                        tracing::info!(
                            "External update received for task {} from {:?}",
                            task_id,
                            source
                        );
                        // TODO: Process external updates
                    }
                    _ => {
                        // Other events are handled elsewhere
                    }
                }
            }
        });
    }

    pub fn get_enabled_integrations(&self) -> Vec<IntegrationType> {
        let mut enabled = Vec::new();

        if self.github.is_some() {
            enabled.push(IntegrationType::GitHub);
        }
        if self.linear.is_some() {
            enabled.push(IntegrationType::Linear);
        }
        if self.slack.is_some() {
            enabled.push(IntegrationType::Slack);
        }

        enabled
    }

    pub fn is_integration_enabled(&self, integration_type: &IntegrationType) -> bool {
        match integration_type {
            IntegrationType::GitHub => self.github.is_some(),
            IntegrationType::Linear => self.linear.is_some(),
            IntegrationType::Slack => self.slack.is_some(),
        }
    }

    pub fn get_config(&self) -> &IntegrationConfig {
        &self.config
    }

    pub async fn sync_task(&self, task_id: TaskId) -> Result<()> {
        if let Some(sync_engine) = &self.sync_engine {
            let mut engine = sync_engine.write().await;
            engine.sync_task(task_id).await?;
        }
        Ok(())
    }

    pub async fn get_sync_stats(&self) -> Option<crate::sync::SyncStats> {
        if let Some(sync_engine) = &self.sync_engine {
            let engine = sync_engine.read().await;
            Some(engine.get_stats().clone())
        } else {
            None
        }
    }

    pub async fn resolve_conflicts(&self, conflicts: Vec<SyncConflict>) -> Result<()> {
        if let Some(sync_engine) = &self.sync_engine {
            let mut engine = sync_engine.write().await;
            engine.resolve_conflicts(conflicts).await?;
        }
        Ok(())
    }
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self {
            github: GitHubConfigWrapper {
                enabled: false,
                config: GitHubConfig {
                    access_token: String::new(),
                    webhook_secret: String::new(),
                    default_repo: String::new(),
                },
            },
            linear: LinearConfigWrapper {
                enabled: false,
                config: LinearConfig {
                    api_key: String::new(),
                    webhook_secret: String::new(),
                    team_id: String::new(),
                    default_project_id: None,
                },
            },
            slack: SlackConfigWrapper {
                enabled: false,
                config: SlackConfig {
                    bot_token: String::new(),
                    signing_secret: String::new(),
                    default_channel: String::new(),
                    notification_levels: vec![NotificationLevel::All],
                },
            },
            sync: SyncConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_config() -> IntegrationConfig {
        IntegrationConfig {
            github: GitHubConfigWrapper {
                enabled: false,
                config: GitHubConfig {
                    access_token: "test_token".to_string(),
                    webhook_secret: "test_secret".to_string(),
                    default_repo: "test/repo".to_string(),
                },
            },
            linear: LinearConfigWrapper {
                enabled: false,
                config: LinearConfig {
                    api_key: "test_key".to_string(),
                    webhook_secret: "test_secret".to_string(),
                    team_id: "test_team".to_string(),
                    default_project_id: Some("test_project".to_string()),
                },
            },
            slack: SlackConfigWrapper {
                enabled: false,
                config: SlackConfig {
                    bot_token: "xoxb-test-token".to_string(),
                    signing_secret: "test_secret".to_string(),
                    default_channel: "#test".to_string(),
                    notification_levels: vec![NotificationLevel::All],
                },
            },
            sync: SyncConfig {
                enabled: true,
                interval_seconds: 300,
                conflict_resolution: ConflictResolutionStrategy::LuceWins,
                auto_create_external_tasks: false,
            },
        }
    }

    #[tokio::test]
    async fn test_integration_manager_creation() {
        let config = create_test_config();
        let manager = IntegrationManager::new(config).await.unwrap();

        assert!(!manager.is_integration_enabled(&IntegrationType::GitHub));
        assert!(!manager.is_integration_enabled(&IntegrationType::Linear));
        assert!(!manager.is_integration_enabled(&IntegrationType::Slack));
    }

    #[tokio::test]
    async fn test_integration_manager_disabled_integrations() {
        let config = IntegrationConfig::default();
        let manager = IntegrationManager::new(config).await.unwrap();

        assert!(!manager.is_integration_enabled(&IntegrationType::GitHub));
        assert!(!manager.is_integration_enabled(&IntegrationType::Linear));
        assert!(!manager.is_integration_enabled(&IntegrationType::Slack));

        assert!(manager.get_enabled_integrations().is_empty());
    }

    #[test]
    fn test_enabled_integrations() {
        let config = create_test_config();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let manager = rt.block_on(IntegrationManager::new(config)).unwrap();

        let enabled = manager.get_enabled_integrations();
        assert_eq!(enabled.len(), 0);
        assert!(enabled.is_empty());
    }

    #[test]
    fn test_webhook_handler_availability() {
        let config = create_test_config();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let manager = rt.block_on(IntegrationManager::new(config)).unwrap();

        assert!(manager.get_webhook_handler().is_none());
        assert!(manager.get_webhook_router().is_none());
    }

    #[test]
    fn test_webhook_handler_unavailable_when_no_integrations() {
        let config = IntegrationConfig::default();
        let rt = tokio::runtime::Runtime::new().unwrap();
        let manager = rt.block_on(IntegrationManager::new(config)).unwrap();

        assert!(manager.get_webhook_handler().is_none());
        assert!(manager.get_webhook_router().is_none());
    }

    #[tokio::test]
    async fn test_task_created_handling() {
        let config = create_test_config();
        let manager = IntegrationManager::new(config).await.unwrap();

        let task_id = Uuid::new_v4();
        let result = manager
            .handle_task_created(task_id, "Test Task", Some("Test description"))
            .await;

        // Should not fail even though external API calls will fail in tests
        if let Err(e) = &result {
            eprintln!("Test failed with error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_task_updated_handling() {
        let config = create_test_config();
        let manager = IntegrationManager::new(config).await.unwrap();

        let task_id = Uuid::new_v4();
        let changes = TaskChanges {
            status: Some("InProgress".to_string()),
            title: None,
            description: None,
            priority: Some("High".to_string()),
            assignee: None,
        };

        let result = manager
            .handle_task_updated(task_id, "Test Task", changes)
            .await;

        // Should not fail even though external API calls will fail in tests
        if let Err(e) = &result {
            eprintln!("Test failed with error: {:?}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_integration_config_serialization() {
        let config = create_test_config();

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: IntegrationConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.github.enabled, deserialized.github.enabled);
        assert_eq!(config.linear.enabled, deserialized.linear.enabled);
        assert_eq!(config.slack.enabled, deserialized.slack.enabled);
        assert_eq!(config.sync.enabled, deserialized.sync.enabled);
    }

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();

        assert!(config.enabled);
        assert_eq!(config.interval_seconds, 300);
        assert!(matches!(
            config.conflict_resolution,
            ConflictResolutionStrategy::LuceWins
        ));
        assert!(!config.auto_create_external_tasks);
    }

    #[tokio::test]
    async fn test_sync_task() {
        let config = create_test_config();
        let manager = IntegrationManager::new(config).await.unwrap();

        let task_id = Uuid::new_v4();
        let result = manager.sync_task(task_id).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_sync_stats() {
        let config = create_test_config();
        let manager = IntegrationManager::new(config).await.unwrap();

        let stats = manager.get_sync_stats().await;
        assert!(stats.is_some());
    }

    #[tokio::test]
    async fn test_resolve_conflicts() {
        let config = create_test_config();
        let manager = IntegrationManager::new(config).await.unwrap();

        let conflicts = vec![];
        let result = manager.resolve_conflicts(conflicts).await;

        assert!(result.is_ok());
    }
}
