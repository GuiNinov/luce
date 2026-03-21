use anyhow::Result;
use luce_shared::TaskId;
use serde::{Deserialize, Serialize};
use tokio::time::{interval, Duration};

use crate::{
    GitHubIntegration, IntegrationEvent, IntegrationEventBus, IntegrationType, LinearIntegration,
    SlackIntegration, SyncConflict, SyncResult,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ConflictResolutionStrategy {
    #[default]
    LuceWins, // Luce is source of truth
    ExternalWins,     // External system is source of truth
    MostRecentWins,   // Most recently updated wins
    ManualResolution, // Require manual conflict resolution
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncStats {
    pub total_tasks: usize,
    pub successful_syncs: usize,
    pub failed_syncs: usize,
    pub conflicts_resolved: usize,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug)]
pub struct SyncEngine {
    github: Option<GitHubIntegration>,
    linear: Option<LinearIntegration>,
    slack: Option<SlackIntegration>,
    event_bus: IntegrationEventBus,
    conflict_resolution: ConflictResolutionStrategy,
    stats: SyncStats,
    sync_interval: Duration,
}

impl SyncEngine {
    pub fn new(
        github: Option<GitHubIntegration>,
        linear: Option<LinearIntegration>,
        slack: Option<SlackIntegration>,
        event_bus: IntegrationEventBus,
    ) -> Self {
        Self {
            github,
            linear,
            slack,
            event_bus,
            conflict_resolution: ConflictResolutionStrategy::default(),
            stats: SyncStats::default(),
            sync_interval: Duration::from_secs(300), // 5 minutes default
        }
    }

    pub fn with_conflict_resolution_strategy(
        mut self,
        strategy: ConflictResolutionStrategy,
    ) -> Self {
        self.conflict_resolution = strategy;
        self
    }

    pub fn with_sync_interval(mut self, interval: Duration) -> Self {
        self.sync_interval = interval;
        self
    }

    pub async fn sync_task(&mut self, task_id: TaskId) -> Result<Vec<SyncResult>> {
        tracing::info!("Starting sync for task: {}", task_id);
        let mut results = Vec::new();

        // Sync with GitHub if enabled
        if let Some(github) = &self.github {
            match self.sync_task_with_github(task_id, github).await {
                Ok(result) => {
                    results.push(result);
                    self.stats.successful_syncs += 1;
                }
                Err(e) => {
                    tracing::error!("Failed to sync task {} with GitHub: {}", task_id, e);
                    results.push(SyncResult {
                        task_id,
                        integration_type: IntegrationType::GitHub,
                        success: false,
                        error: Some(e.to_string()),
                        timestamp: chrono::Utc::now(),
                    });
                    self.stats.failed_syncs += 1;
                }
            }
        }

        // Sync with Linear if enabled
        if let Some(linear) = &self.linear {
            match self.sync_task_with_linear(task_id, linear).await {
                Ok(result) => {
                    results.push(result);
                    self.stats.successful_syncs += 1;
                }
                Err(e) => {
                    tracing::error!("Failed to sync task {} with Linear: {}", task_id, e);
                    results.push(SyncResult {
                        task_id,
                        integration_type: IntegrationType::Linear,
                        success: false,
                        error: Some(e.to_string()),
                        timestamp: chrono::Utc::now(),
                    });
                    self.stats.failed_syncs += 1;
                }
            }
        }

        // Notify Slack of sync completion if enabled
        if let Some(slack) = &self.slack {
            if let Err(e) = self.notify_sync_completion(task_id, &results, slack).await {
                tracing::warn!("Failed to notify Slack of sync completion: {}", e);
            }
        }

        self.stats.last_sync = Some(chrono::Utc::now());

        // Publish sync event
        let event = IntegrationEvent::ExternalUpdate {
            task_id,
            source: IntegrationType::GitHub, // This should be more generic
            data: serde_json::to_value(&results)?,
        };

        if let Err(e) = self.event_bus.publish(event).await {
            tracing::warn!("Failed to publish sync event: {}", e);
        }

        tracing::info!(
            "Completed sync for task: {} with {} results",
            task_id,
            results.len()
        );
        Ok(results)
    }

    async fn sync_task_with_github(
        &self,
        task_id: TaskId,
        _github: &GitHubIntegration,
    ) -> Result<SyncResult> {
        // TODO: Implement actual GitHub sync logic
        // This would involve:
        // 1. Check if task has GitHub metadata
        // 2. If yes, fetch GitHub issue and compare with task
        // 3. If no, optionally create GitHub issue
        // 4. Resolve any conflicts
        // 5. Update both systems as needed

        tracing::debug!("Syncing task {} with GitHub", task_id);

        // Placeholder implementation
        Ok(SyncResult {
            task_id,
            integration_type: IntegrationType::GitHub,
            success: true,
            error: None,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn sync_task_with_linear(
        &self,
        task_id: TaskId,
        _linear: &LinearIntegration,
    ) -> Result<SyncResult> {
        // TODO: Implement actual Linear sync logic
        // This would involve:
        // 1. Check if task has Linear metadata
        // 2. If yes, fetch Linear issue and compare with task
        // 3. If no, optionally create Linear issue
        // 4. Resolve any conflicts
        // 5. Update both systems as needed

        tracing::debug!("Syncing task {} with Linear", task_id);

        // Placeholder implementation
        Ok(SyncResult {
            task_id,
            integration_type: IntegrationType::Linear,
            success: true,
            error: None,
            timestamp: chrono::Utc::now(),
        })
    }

    async fn notify_sync_completion(
        &self,
        task_id: TaskId,
        results: &[SyncResult],
        slack: &SlackIntegration,
    ) -> Result<()> {
        let successful_count = results.iter().filter(|r| r.success).count();
        let failed_count = results.len() - successful_count;

        if failed_count > 0 {
            // Notify about sync failures
            slack
                .notify_task_failed(
                    task_id,
                    &format!("Sync failed for task {}", task_id),
                    &format!("{} integrations failed to sync", failed_count),
                )
                .await?;
        }

        tracing::debug!("Notified Slack about sync completion for task {}", task_id);
        Ok(())
    }

    pub async fn resolve_conflicts(&mut self, conflicts: Vec<SyncConflict>) -> Result<()> {
        if conflicts.is_empty() {
            return Ok(());
        }

        tracing::info!("Resolving {} conflicts", conflicts.len());

        for conflict in conflicts {
            match self.conflict_resolution {
                ConflictResolutionStrategy::LuceWins => {
                    self.resolve_conflict_luce_wins(&conflict).await?;
                }
                ConflictResolutionStrategy::ExternalWins => {
                    self.resolve_conflict_external_wins(&conflict).await?;
                }
                ConflictResolutionStrategy::MostRecentWins => {
                    self.resolve_conflict_most_recent_wins(&conflict).await?;
                }
                ConflictResolutionStrategy::ManualResolution => {
                    self.queue_manual_resolution(&conflict).await?;
                }
            }
            self.stats.conflicts_resolved += 1;
        }

        tracing::info!("Resolved all conflicts");
        Ok(())
    }

    async fn resolve_conflict_luce_wins(&self, conflict: &SyncConflict) -> Result<()> {
        tracing::info!(
            "Resolving conflict for field '{}' with Luce value winning",
            conflict.field
        );
        // TODO: Update external system with Luce value
        Ok(())
    }

    async fn resolve_conflict_external_wins(&self, conflict: &SyncConflict) -> Result<()> {
        tracing::info!(
            "Resolving conflict for field '{}' with external value winning",
            conflict.field
        );
        // TODO: Update Luce with external value
        Ok(())
    }

    async fn resolve_conflict_most_recent_wins(&self, _conflict: &SyncConflict) -> Result<()> {
        tracing::info!("Resolving conflict with most recent value winning");
        // TODO: Compare timestamps and use most recent value
        Ok(())
    }

    async fn queue_manual_resolution(&self, conflict: &SyncConflict) -> Result<()> {
        tracing::info!("Queueing conflict for manual resolution");

        // Publish conflict event for manual intervention
        let event = IntegrationEvent::SyncConflict {
            task_id: conflict.task_id,
            conflicts: vec![conflict.clone()],
        };

        let _ = self.event_bus.publish(event).await;

        // TODO: Store conflict in a queue for manual resolution UI
        Ok(())
    }

    pub async fn start_periodic_sync(&mut self) {
        tracing::info!(
            "Starting periodic sync with interval: {:?}",
            self.sync_interval
        );
        let mut interval = interval(self.sync_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.sync_all_tasks().await {
                tracing::error!("Periodic sync failed: {}", e);
            }
        }
    }

    async fn sync_all_tasks(&mut self) -> Result<()> {
        tracing::info!("Starting sync of all tasks");

        // TODO: Get all tasks from the task graph
        // This would require access to the TaskGraph, which we don't have here
        // In a real implementation, this would be injected or accessed via a service

        let sample_task_ids = vec![]; // Placeholder

        for task_id in sample_task_ids {
            if let Err(e) = self.sync_task(task_id).await {
                tracing::error!("Failed to sync task {}: {}", task_id, e);
            }

            // Small delay between syncs to avoid rate limiting
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        tracing::info!("Completed sync of all tasks");
        Ok(())
    }

    pub fn get_stats(&self) -> &SyncStats {
        &self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = SyncStats::default();
    }

    pub fn set_conflict_resolution_strategy(&mut self, strategy: ConflictResolutionStrategy) {
        self.conflict_resolution = strategy;
    }

    pub fn get_conflict_resolution_strategy(&self) -> &ConflictResolutionStrategy {
        &self.conflict_resolution
    }

    pub fn is_integration_enabled(&self, integration_type: &IntegrationType) -> bool {
        match integration_type {
            IntegrationType::GitHub => self.github.is_some(),
            IntegrationType::Linear => self.linear.is_some(),
            IntegrationType::Slack => self.slack.is_some(),
        }
    }

    pub fn enabled_integrations(&self) -> Vec<IntegrationType> {
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
}

// Helper function to detect conflicts between values
pub fn detect_conflict(
    task_id: TaskId,
    field: &str,
    luce_value: &serde_json::Value,
    external_value: &serde_json::Value,
    integration_type: IntegrationType,
) -> Option<SyncConflict> {
    if luce_value != external_value {
        Some(SyncConflict {
            task_id,
            field: field.to_string(),
            luce_value: luce_value.clone(),
            external_value: external_value.clone(),
            integration_type,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GitHubConfig, LinearConfig, NotificationLevel, SlackConfig};
    use uuid::Uuid;

    fn create_test_sync_engine() -> SyncEngine {
        let event_bus = IntegrationEventBus::new();

        let github_config = GitHubConfig {
            access_token: "test_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            default_repo: "test/repo".to_string(),
        };

        let linear_config = LinearConfig {
            api_key: "test_key".to_string(),
            webhook_secret: "test_secret".to_string(),
            team_id: "test_team".to_string(),
            default_project_id: None,
        };

        let slack_config = SlackConfig {
            bot_token: "xoxb-test-token".to_string(),
            signing_secret: "test_secret".to_string(),
            default_channel: "#test".to_string(),
            notification_levels: vec![NotificationLevel::All],
        };

        SyncEngine::new(
            Some(GitHubIntegration::new(github_config, event_bus.clone())),
            Some(LinearIntegration::new(linear_config, event_bus.clone())),
            Some(SlackIntegration::new(slack_config, event_bus.clone())),
            event_bus,
        )
    }

    #[test]
    fn test_sync_engine_creation() {
        let engine = create_test_sync_engine();
        assert!(engine.is_integration_enabled(&IntegrationType::GitHub));
        assert!(engine.is_integration_enabled(&IntegrationType::Linear));
        assert!(engine.is_integration_enabled(&IntegrationType::Slack));
    }

    #[test]
    fn test_enabled_integrations() {
        let engine = create_test_sync_engine();
        let enabled = engine.enabled_integrations();

        assert_eq!(enabled.len(), 3);
        assert!(enabled.contains(&IntegrationType::GitHub));
        assert!(enabled.contains(&IntegrationType::Linear));
        assert!(enabled.contains(&IntegrationType::Slack));
    }

    #[test]
    fn test_conflict_resolution_strategy() {
        let mut engine = create_test_sync_engine();

        assert!(matches!(
            engine.get_conflict_resolution_strategy(),
            ConflictResolutionStrategy::LuceWins
        ));

        engine.set_conflict_resolution_strategy(ConflictResolutionStrategy::ExternalWins);
        assert!(matches!(
            engine.get_conflict_resolution_strategy(),
            ConflictResolutionStrategy::ExternalWins
        ));
    }

    #[test]
    fn test_with_methods() {
        let event_bus = IntegrationEventBus::new();
        let engine = SyncEngine::new(None, None, None, event_bus)
            .with_conflict_resolution_strategy(ConflictResolutionStrategy::MostRecentWins)
            .with_sync_interval(Duration::from_secs(60));

        assert!(matches!(
            engine.get_conflict_resolution_strategy(),
            ConflictResolutionStrategy::MostRecentWins
        ));
        assert_eq!(engine.sync_interval, Duration::from_secs(60));
    }

    #[test]
    fn test_stats_management() {
        let mut engine = create_test_sync_engine();

        engine.stats.successful_syncs = 5;
        engine.stats.failed_syncs = 2;
        engine.stats.conflicts_resolved = 1;

        let stats = engine.get_stats();
        assert_eq!(stats.successful_syncs, 5);
        assert_eq!(stats.failed_syncs, 2);
        assert_eq!(stats.conflicts_resolved, 1);

        engine.reset_stats();
        let stats = engine.get_stats();
        assert_eq!(stats.successful_syncs, 0);
        assert_eq!(stats.failed_syncs, 0);
        assert_eq!(stats.conflicts_resolved, 0);
    }

    #[test]
    fn test_detect_conflict() {
        let task_id = Uuid::new_v4();
        let luce_value = serde_json::json!("Completed");
        let external_value = serde_json::json!("Done");

        let conflict = detect_conflict(
            task_id,
            "status",
            &luce_value,
            &external_value,
            IntegrationType::Linear,
        );

        assert!(conflict.is_some());
        let conflict = conflict.unwrap();
        assert_eq!(conflict.field, "status");
        assert_eq!(conflict.luce_value, luce_value);
        assert_eq!(conflict.external_value, external_value);
        assert!(matches!(conflict.integration_type, IntegrationType::Linear));
    }

    #[test]
    fn test_no_conflict_detected() {
        let task_id = Uuid::new_v4();
        let same_value = serde_json::json!("Completed");

        let conflict = detect_conflict(
            task_id,
            "status",
            &same_value,
            &same_value,
            IntegrationType::GitHub,
        );

        assert!(conflict.is_none());
    }

    #[test]
    fn test_sync_stats_serialization() {
        let stats = SyncStats {
            total_tasks: 10,
            successful_syncs: 8,
            failed_syncs: 2,
            conflicts_resolved: 1,
            last_sync: Some(chrono::Utc::now()),
        };

        let serialized = serde_json::to_string(&stats).unwrap();
        let deserialized: SyncStats = serde_json::from_str(&serialized).unwrap();

        assert_eq!(stats.total_tasks, deserialized.total_tasks);
        assert_eq!(stats.successful_syncs, deserialized.successful_syncs);
        assert_eq!(stats.failed_syncs, deserialized.failed_syncs);
        assert_eq!(stats.conflicts_resolved, deserialized.conflicts_resolved);
    }

    #[tokio::test]
    async fn test_sync_task() {
        let mut engine = create_test_sync_engine();
        let task_id = Uuid::new_v4();

        let results = engine.sync_task(task_id).await.unwrap();

        // Should have results for GitHub and Linear (Slack doesn't return sync results)
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.success));
        assert!(results
            .iter()
            .any(|r| matches!(r.integration_type, IntegrationType::GitHub)));
        assert!(results
            .iter()
            .any(|r| matches!(r.integration_type, IntegrationType::Linear)));
    }

    #[tokio::test]
    async fn test_resolve_conflicts_empty() {
        let mut engine = create_test_sync_engine();
        let result = engine.resolve_conflicts(vec![]).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_conflict_resolution_strategy_serialization() {
        let strategies = vec![
            ConflictResolutionStrategy::LuceWins,
            ConflictResolutionStrategy::ExternalWins,
            ConflictResolutionStrategy::MostRecentWins,
            ConflictResolutionStrategy::ManualResolution,
        ];

        for strategy in strategies {
            let serialized = serde_json::to_string(&strategy).unwrap();
            let deserialized: ConflictResolutionStrategy =
                serde_json::from_str(&serialized).unwrap();

            match (&strategy, &deserialized) {
                (ConflictResolutionStrategy::LuceWins, ConflictResolutionStrategy::LuceWins) => {}
                (
                    ConflictResolutionStrategy::ExternalWins,
                    ConflictResolutionStrategy::ExternalWins,
                ) => {}
                (
                    ConflictResolutionStrategy::MostRecentWins,
                    ConflictResolutionStrategy::MostRecentWins,
                ) => {}
                (
                    ConflictResolutionStrategy::ManualResolution,
                    ConflictResolutionStrategy::ManualResolution,
                ) => {}
                _ => panic!("Serialization/deserialization mismatch"),
            }
        }
    }
}
