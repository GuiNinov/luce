use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use luce_shared::LuceConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::services::TaskService;

#[derive(Serialize, Deserialize)]
pub struct IntegrationStatus {
    pub name: String,
    pub enabled: bool,
    pub configured: bool,
    pub valid: bool,
    pub details: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct IntegrationsResponse {
    pub integrations: Vec<IntegrationStatus>,
    pub enabled_count: usize,
}

#[derive(Serialize, Deserialize)]
pub struct WebhookRequest {
    pub url: String,
    pub events: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct WebhookResponse {
    pub id: String,
    pub url: String,
    pub events: Vec<String>,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct GitHubSyncRequest {
    pub repository: Option<String>,
    pub include_issues: Option<bool>,
    pub include_prs: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct GitHubSyncResponse {
    pub repository: String,
    pub issues_synced: usize,
    pub prs_synced: usize,
    pub tasks_created: usize,
    pub tasks_updated: usize,
}

pub async fn list_integrations(
    Extension(service): Extension<Arc<TaskService>>,
) -> Result<Json<IntegrationsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let config = LuceConfig::from_env().unwrap_or_default();

    let mut integrations = Vec::new();

    // GitHub Integration
    let github_status = if let Some(github_config) = &config.integrations.github {
        IntegrationStatus {
            name: "github".to_string(),
            enabled: true,
            configured: true,
            valid: github_config.is_valid(),
            details: Some(serde_json::json!({
                "repository": github_config.default_repo,
                "webhook_url": github_config.webhook_url
            })),
        }
    } else {
        IntegrationStatus {
            name: "github".to_string(),
            enabled: false,
            configured: false,
            valid: false,
            details: None,
        }
    };
    integrations.push(github_status);

    // Slack Integration
    let slack_status = if let Some(slack_config) = &config.integrations.slack {
        IntegrationStatus {
            name: "slack".to_string(),
            enabled: true,
            configured: true,
            valid: slack_config.is_valid(),
            details: Some(serde_json::json!({
                "default_channel": slack_config.default_channel
            })),
        }
    } else {
        IntegrationStatus {
            name: "slack".to_string(),
            enabled: false,
            configured: false,
            valid: false,
            details: None,
        }
    };
    integrations.push(slack_status);

    // Linear Integration
    let linear_status = if let Some(linear_config) = &config.integrations.linear {
        IntegrationStatus {
            name: "linear".to_string(),
            enabled: true,
            configured: true,
            valid: linear_config.is_valid(),
            details: Some(serde_json::json!({
                "team_id": linear_config.team_id
            })),
        }
    } else {
        IntegrationStatus {
            name: "linear".to_string(),
            enabled: false,
            configured: false,
            valid: false,
            details: None,
        }
    };
    integrations.push(linear_status);

    let enabled_count = integrations.iter().filter(|i| i.enabled && i.valid).count();

    Ok(Json(IntegrationsResponse {
        integrations,
        enabled_count,
    }))
}

pub async fn test_integration(
    Extension(service): Extension<Arc<TaskService>>,
    Path(integration_name): Path<String>,
) -> Result<Json<IntegrationStatus>, (StatusCode, Json<serde_json::Value>)> {
    let config = LuceConfig::from_env().unwrap_or_default();

    let status = match integration_name.as_str() {
        "github" => {
            if let Some(github_config) = &config.integrations.github {
                // TODO: Actually test GitHub API connection
                IntegrationStatus {
                    name: "github".to_string(),
                    enabled: true,
                    configured: true,
                    valid: github_config.is_valid(),
                    details: Some(serde_json::json!({
                        "repository": github_config.default_repo,
                        "test_result": "Connection test not implemented yet"
                    })),
                }
            } else {
                IntegrationStatus {
                    name: "github".to_string(),
                    enabled: false,
                    configured: false,
                    valid: false,
                    details: Some(serde_json::json!({
                        "error": "GitHub integration not configured"
                    })),
                }
            }
        }
        "slack" => {
            if let Some(slack_config) = &config.integrations.slack {
                // TODO: Actually test Slack API connection
                IntegrationStatus {
                    name: "slack".to_string(),
                    enabled: true,
                    configured: true,
                    valid: slack_config.is_valid(),
                    details: Some(serde_json::json!({
                        "test_result": "Connection test not implemented yet"
                    })),
                }
            } else {
                IntegrationStatus {
                    name: "slack".to_string(),
                    enabled: false,
                    configured: false,
                    valid: false,
                    details: Some(serde_json::json!({
                        "error": "Slack integration not configured"
                    })),
                }
            }
        }
        "linear" => {
            if let Some(linear_config) = &config.integrations.linear {
                // TODO: Actually test Linear API connection
                IntegrationStatus {
                    name: "linear".to_string(),
                    enabled: true,
                    configured: true,
                    valid: linear_config.is_valid(),
                    details: Some(serde_json::json!({
                        "team_id": linear_config.team_id,
                        "test_result": "Connection test not implemented yet"
                    })),
                }
            } else {
                IntegrationStatus {
                    name: "linear".to_string(),
                    enabled: false,
                    configured: false,
                    valid: false,
                    details: Some(serde_json::json!({
                        "error": "Linear integration not configured"
                    })),
                }
            }
        }
        _ => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("Unknown integration: {}", integration_name)
                })),
            ));
        }
    };

    Ok(Json(status))
}

pub async fn create_github_webhook(
    Extension(service): Extension<Arc<TaskService>>,
    Json(request): Json<WebhookRequest>,
) -> Result<(StatusCode, Json<WebhookResponse>), (StatusCode, Json<serde_json::Value>)> {
    let config = LuceConfig::from_env().unwrap_or_default();

    let github_config = config.integrations.github.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "GitHub integration not configured"
            })),
        )
    })?;

    if !github_config.is_valid() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "GitHub configuration is invalid"
            })),
        ));
    }

    // TODO: Implement actual webhook creation with GitHub API
    let webhook_response = WebhookResponse {
        id: uuid::Uuid::new_v4().to_string(),
        url: request.url,
        events: request.events.unwrap_or_else(|| {
            vec![
                "issues".to_string(),
                "pull_request".to_string(),
                "issue_comment".to_string(),
            ]
        }),
        active: true,
        created_at: chrono::Utc::now(),
    };

    println!(
        "Created GitHub webhook for repository: {}",
        github_config.default_repo
    );

    Ok((StatusCode::CREATED, Json(webhook_response)))
}

pub async fn sync_github_repository(
    Extension(service): Extension<Arc<TaskService>>,
    Json(request): Json<GitHubSyncRequest>,
) -> Result<Json<GitHubSyncResponse>, (StatusCode, Json<serde_json::Value>)> {
    let config = LuceConfig::from_env().unwrap_or_default();

    let github_config = config.integrations.github.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "GitHub integration not configured"
            })),
        )
    })?;

    if !github_config.is_valid() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "GitHub configuration is invalid"
            })),
        ));
    }

    let repository = request
        .repository
        .unwrap_or_else(|| github_config.default_repo.clone());
    let include_issues = request.include_issues.unwrap_or(true);
    let include_prs = request.include_prs.unwrap_or(true);

    // TODO: Implement actual GitHub synchronization
    println!("Syncing GitHub repository: {}", repository);
    println!("Include issues: {}", include_issues);
    println!("Include PRs: {}", include_prs);

    let sync_response = GitHubSyncResponse {
        repository,
        issues_synced: 0, // Placeholder values
        prs_synced: 0,
        tasks_created: 0,
        tasks_updated: 0,
    };

    Ok(Json(sync_response))
}

pub async fn handle_github_webhook(
    Extension(service): Extension<Arc<TaskService>>,
    headers: axum::http::HeaderMap,
    body: String,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // TODO: Verify webhook signature
    // let signature = headers.get("X-Hub-Signature-256")
    //     .and_then(|v| v.to_str().ok())
    //     .ok_or_else(|| {
    //         (StatusCode::BAD_REQUEST, Json(serde_json::json!({
    //             "error": "Missing webhook signature"
    //         })))
    //     })?;

    // TODO: Parse webhook payload and handle events
    println!("Received GitHub webhook");
    println!("Headers: {:?}", headers.keys().collect::<Vec<_>>());
    println!("Body length: {}", body.len());

    Ok(Json(serde_json::json!({
        "status": "received",
        "message": "Webhook processing not implemented yet"
    })))
}

pub fn integration_routes() -> Router {
    Router::new()
        .route("/integrations", get(list_integrations))
        .route("/integrations/:name/test", post(test_integration))
        .route("/integrations/github/webhook", post(create_github_webhook))
        .route("/integrations/github/sync", post(sync_github_repository))
        .route("/integrations/github/webhooks", post(handle_github_webhook))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_webhook_request_serialization() {
        let request = WebhookRequest {
            url: "https://example.com/webhook".to_string(),
            events: Some(vec!["issues".to_string(), "pull_request".to_string()]),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: WebhookRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.url, deserialized.url);
        assert_eq!(request.events, deserialized.events);
    }

    #[tokio::test]
    async fn test_github_sync_request_serialization() {
        let request = GitHubSyncRequest {
            repository: Some("owner/repo".to_string()),
            include_issues: Some(true),
            include_prs: Some(false),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: GitHubSyncRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.repository, deserialized.repository);
        assert_eq!(request.include_issues, deserialized.include_issues);
        assert_eq!(request.include_prs, deserialized.include_prs);
    }

    #[test]
    fn test_integration_status_serialization() {
        let status = IntegrationStatus {
            name: "github".to_string(),
            enabled: true,
            configured: true,
            valid: true,
            details: Some(json!({
                "repository": "owner/repo"
            })),
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: IntegrationStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(status.name, deserialized.name);
        assert_eq!(status.enabled, deserialized.enabled);
        assert_eq!(status.configured, deserialized.configured);
        assert_eq!(status.valid, deserialized.valid);
    }

    #[test]
    fn test_integrations_response_serialization() {
        let integrations = vec![
            IntegrationStatus {
                name: "github".to_string(),
                enabled: true,
                configured: true,
                valid: true,
                details: None,
            },
            IntegrationStatus {
                name: "slack".to_string(),
                enabled: false,
                configured: false,
                valid: false,
                details: None,
            },
        ];

        let response = IntegrationsResponse {
            integrations: integrations.clone(),
            enabled_count: 1,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: IntegrationsResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.enabled_count, deserialized.enabled_count);
        assert_eq!(response.integrations.len(), deserialized.integrations.len());
    }
}
