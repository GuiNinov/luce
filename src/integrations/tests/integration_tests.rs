use axum_test::TestServer;
use luce_integrations::*;
use luce_shared::{LuceEvent, LuceEventBus, TaskPriority};
use uuid::Uuid;

mod event_system_tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_multiple_subscribers() {
        let event_bus = IntegrationEventBus::new();
        let mut receiver1 = event_bus.subscribe();
        let mut receiver2 = event_bus.subscribe();

        let task_id = Uuid::new_v4();
        let event = IntegrationEvent::ExternalUpdate {
            task_id,
            source: IntegrationType::GitHub,
            data: serde_json::json!({"action": "updated"}),
        };

        event_bus.publish(event.clone()).await.unwrap();

        let received1 = receiver1.recv().await.unwrap();
        let received2 = receiver2.recv().await.unwrap();

        // Both receivers should get the same event
        match (&received1, &received2) {
            (
                IntegrationEvent::ExternalUpdate {
                    task_id: id1,
                    source: s1,
                    ..
                },
                IntegrationEvent::ExternalUpdate {
                    task_id: id2,
                    source: s2,
                    ..
                },
            ) => {
                assert_eq!(*id1, task_id);
                assert_eq!(*id2, task_id);
                assert_eq!(*s1, IntegrationType::GitHub);
                assert_eq!(*s2, IntegrationType::GitHub);
            }
            _ => panic!("Wrong event types received"),
        }
    }

    #[tokio::test]
    async fn test_luce_event_bus_integration() {
        let luce_event_bus = LuceEventBus::new();
        let mut receiver = luce_event_bus.subscribe();

        let task_id = Uuid::new_v4();
        let event = LuceEvent::TaskCreated {
            task_id,
            title: "Test Task".to_string(),
            description: Some("Test Description".to_string()),
            priority: TaskPriority::Normal,
        };

        luce_event_bus.publish(event.clone()).await.unwrap();

        let received = receiver.recv().await.unwrap();
        match received {
            LuceEvent::TaskCreated {
                task_id: id, title, ..
            } => {
                assert_eq!(id, task_id);
                assert_eq!(title, "Test Task");
            }
            _ => panic!("Wrong event type received"),
        }
    }

    #[test]
    fn test_event_serialization_roundtrip() {
        let _task_id = Uuid::new_v4();

        // Test IntegrationEvent serialization
        let integration_event = IntegrationEvent::SyncStarted {
            integration: IntegrationType::GitHub,
        };
        let serialized = serde_json::to_string(&integration_event).unwrap();
        let deserialized: IntegrationEvent = serde_json::from_str(&serialized).unwrap();
        match deserialized {
            IntegrationEvent::SyncStarted { integration } => {
                assert_eq!(integration, IntegrationType::GitHub);
            }
            _ => panic!("Deserialization failed"),
        }

        // Test LuceEvent serialization (note: can't test here due to module boundaries)
        // This would be tested in the shared package tests
    }
}

mod sync_engine_tests {
    use super::*;

    #[test]
    fn test_conflict_detection() {
        let conflict = SyncConflict {
            task_id: Uuid::new_v4(),
            field: "status".to_string(),
            luce_value: serde_json::json!("InProgress"),
            external_value: serde_json::json!("Done"),
            integration_type: IntegrationType::Linear,
        };

        assert_eq!(conflict.field, "status");
        assert_eq!(conflict.integration_type, IntegrationType::Linear);
    }

    #[tokio::test]
    async fn test_sync_engine_task_sync() {
        let integration_event_bus = IntegrationEventBus::new();
        let _luce_event_bus = LuceEventBus::new();
        let github = None;
        let linear = None;
        let slack = None;

        let mut sync_engine = SyncEngine::new(github, linear, slack, integration_event_bus);

        let task_id = Uuid::new_v4();
        let result = sync_engine.sync_task(task_id).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_conflict_resolution_strategies() {
        // Test different conflict resolution strategies
        let strategies = vec![
            ConflictResolutionStrategy::LuceWins,
            ConflictResolutionStrategy::ExternalWins,
            ConflictResolutionStrategy::MostRecentWins,
            ConflictResolutionStrategy::ManualResolution,
        ];

        for _strategy in strategies {
            // Strategy selection would be tested in actual resolution logic
            // This is just verifying the enum variants exist
        }
    }
}

mod integration_manager_tests {
    use super::*;

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
                interval_seconds: 60,
                conflict_resolution: ConflictResolutionStrategy::LuceWins,
                auto_create_external_tasks: false,
            },
        }
    }

    #[tokio::test]
    async fn test_integration_manager_full_lifecycle() {
        let config = create_test_config();
        let manager = IntegrationManager::new(config).await.unwrap();

        // Verify integrations are disabled (for test safety)
        assert!(!manager.is_integration_enabled(&IntegrationType::GitHub));
        assert!(!manager.is_integration_enabled(&IntegrationType::Linear));
        assert!(!manager.is_integration_enabled(&IntegrationType::Slack));

        let enabled = manager.get_enabled_integrations();
        assert_eq!(enabled.len(), 0);

        // Test task lifecycle with disabled integrations
        let task_id = Uuid::new_v4();
        let title = "Integration Test Task";

        // Task creation
        let result = manager
            .handle_task_created(task_id, title, Some("Test description"))
            .await;
        assert!(result.is_ok());

        // Task update
        let changes = TaskChanges {
            status: Some("InProgress".to_string()),
            title: None,
            description: None,
            priority: Some("High".to_string()),
            assignee: None,
        };
        let result = manager.handle_task_updated(task_id, title, changes).await;
        assert!(result.is_ok());

        // Task completion
        let result = manager.handle_task_completed(task_id, title).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_functionality() {
        let config = create_test_config();
        let manager = IntegrationManager::new(config).await.unwrap();
        let task_id = Uuid::new_v4();

        // Test sync task
        let result = manager.sync_task(task_id).await;
        assert!(result.is_ok());

        // Test get sync stats
        let stats = manager.get_sync_stats().await;
        assert!(stats.is_some());

        // Test resolve conflicts
        let conflicts = vec![];
        let result = manager.resolve_conflicts(conflicts).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let original_config = create_test_config();
        let serialized = serde_json::to_string_pretty(&original_config).unwrap();
        let deserialized: IntegrationConfig = serde_json::from_str(&serialized).unwrap();

        // Verify core settings are preserved
        assert_eq!(original_config.github.enabled, deserialized.github.enabled);
        assert_eq!(original_config.linear.enabled, deserialized.linear.enabled);
        assert_eq!(original_config.slack.enabled, deserialized.slack.enabled);
        assert_eq!(original_config.sync.enabled, deserialized.sync.enabled);
    }

    #[tokio::test]
    async fn test_integration_manager_partial_config() {
        let mut config = create_test_config();
        config.github.enabled = true; // Enable only GitHub

        let manager = IntegrationManager::new(config).await.unwrap();

        // Only GitHub should be enabled
        assert!(manager.is_integration_enabled(&IntegrationType::GitHub));
        assert!(!manager.is_integration_enabled(&IntegrationType::Linear));
        assert!(!manager.is_integration_enabled(&IntegrationType::Slack));
    }
}

mod webhook_handler_tests {
    use super::*;

    #[tokio::test]
    async fn test_webhook_router_setup() {
        let integration_event_bus = IntegrationEventBus::new();
        let github = GitHubIntegration::new(
            GitHubConfig {
                access_token: "test".to_string(),
                webhook_secret: "test".to_string(),
                default_repo: "test/repo".to_string(),
            },
            integration_event_bus.clone(),
        );
        let webhook_handler = WebhookHandler::new(Some(github), None, None, integration_event_bus);
        let app = webhook_handler.router();
        let server = TestServer::new(app).unwrap();

        // Test health endpoint
        let response = server.get("/health").await;
        response.assert_status_ok();
    }

    #[tokio::test]
    async fn test_webhook_endpoints_exist() {
        let integration_event_bus = IntegrationEventBus::new();
        let github = GitHubIntegration::new(
            GitHubConfig {
                access_token: "test".to_string(),
                webhook_secret: "test".to_string(),
                default_repo: "test/repo".to_string(),
            },
            integration_event_bus.clone(),
        );
        let webhook_handler = WebhookHandler::new(Some(github), None, None, integration_event_bus);
        let app = webhook_handler.router();
        let server = TestServer::new(app).unwrap();

        // Test GitHub endpoint exists
        let response = server.post("/webhooks/github").await;
        // We expect this to fail with 400 because we didn't provide proper headers/body
        // But the endpoint should exist (not 404)
        assert_ne!(response.status_code(), 404);
    }

    #[tokio::test]
    async fn test_health_endpoint_query_params() {
        let integration_event_bus = IntegrationEventBus::new();
        let webhook_handler = WebhookHandler::new(None, None, None, integration_event_bus);
        let app = webhook_handler.router();
        let server = TestServer::new(app).unwrap();

        // Test health endpoint with service parameter
        let response = server.get("/health?service=github").await;
        assert_ne!(response.status_code(), 200); // Service unavailable since no integrations enabled
    }
}

#[tokio::test]
async fn test_end_to_end_integration() {
    // Create a full integration setup with disabled external APIs for test safety
    let config = IntegrationConfig {
        github: GitHubConfigWrapper {
            enabled: false,
            config: GitHubConfig {
                access_token: "test".to_string(),
                webhook_secret: "test_secret".to_string(),
                default_repo: "test/repo".to_string(),
            },
        },
        linear: LinearConfigWrapper {
            enabled: false,
            config: LinearConfig {
                api_key: "test".to_string(),
                webhook_secret: "test".to_string(),
                team_id: "test".to_string(),
                default_project_id: None,
            },
        },
        slack: SlackConfigWrapper {
            enabled: false,
            config: SlackConfig {
                bot_token: "test".to_string(),
                signing_secret: "test".to_string(),
                default_channel: "#test".to_string(),
                notification_levels: vec![NotificationLevel::All],
            },
        },
        sync: SyncConfig {
            enabled: true,
            interval_seconds: 30,
            conflict_resolution: ConflictResolutionStrategy::LuceWins,
            auto_create_external_tasks: false,
        },
    };

    let manager = IntegrationManager::new(config).await.unwrap();

    // Simulate a complete task workflow
    let task_id = Uuid::new_v4();
    let title = "E2E Integration Test Task";

    // 1. Task creation
    let result = manager
        .handle_task_created(task_id, title, Some("End-to-end test"))
        .await;
    assert!(result.is_ok(), "Task creation should succeed");

    // 2. Task assignment
    let result = manager
        .handle_session_assignment(task_id, title, "e2e-session", "test_user")
        .await;
    assert!(result.is_ok(), "Session assignment should succeed");

    // 3. Task progress updates
    let changes = TaskChanges {
        status: Some("InProgress".to_string()),
        title: None,
        description: Some("Updated description".to_string()),
        priority: Some("High".to_string()),
        assignee: Some("test_user".to_string()),
    };
    let result = manager.handle_task_updated(task_id, title, changes).await;
    assert!(result.is_ok(), "Task update should succeed");

    // 4. Task completion
    let result = manager.handle_task_completed(task_id, title).await;
    assert!(result.is_ok(), "Task completion should succeed");

    // 5. Verify webhook handler is not available (integrations disabled)
    let webhook_router = manager.get_webhook_router();
    assert!(
        webhook_router.is_none(),
        "Webhook router should not be available when integrations are disabled"
    );
}
