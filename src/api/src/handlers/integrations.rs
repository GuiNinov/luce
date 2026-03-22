use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use luce_shared::{IntegrationType, CredentialData};
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
    let mut integrations = Vec::new();

    // GitHub Integration - check for stored credentials
    let github_status = match service.credential_service.has_active_credentials_for_type(IntegrationType::GitHub).await {
        Ok(true) => {
            // Get the first active GitHub credential for details
            match service.credential_service.get_first_active_credential_for_type(IntegrationType::GitHub).await {
                Ok(Some(credential)) => IntegrationStatus {
                    name: "github".to_string(),
                    enabled: true,
                    configured: true,
                    valid: true,
                    details: Some(serde_json::json!({
                        "credential_name": credential.name,
                        "credential_id": credential.id,
                        "last_used": credential.last_used_at
                    })),
                },
                _ => IntegrationStatus {
                    name: "github".to_string(),
                    enabled: false,
                    configured: false,
                    valid: false,
                    details: Some(serde_json::json!({
                        "error": "Failed to retrieve credential details"
                    })),
                }
            }
        },
        _ => IntegrationStatus {
            name: "github".to_string(),
            enabled: false,
            configured: false,
            valid: false,
            details: Some(serde_json::json!({
                "error": "No active GitHub credentials found"
            })),
        }
    };
    integrations.push(github_status);

    // Slack Integration - check for stored credentials
    let slack_status = match service.credential_service.has_active_credentials_for_type(IntegrationType::Slack).await {
        Ok(true) => {
            match service.credential_service.get_first_active_credential_for_type(IntegrationType::Slack).await {
                Ok(Some(credential)) => IntegrationStatus {
                    name: "slack".to_string(),
                    enabled: true,
                    configured: true,
                    valid: true,
                    details: Some(serde_json::json!({
                        "credential_name": credential.name,
                        "credential_id": credential.id,
                        "last_used": credential.last_used_at
                    })),
                },
                _ => IntegrationStatus {
                    name: "slack".to_string(),
                    enabled: false,
                    configured: false,
                    valid: false,
                    details: Some(serde_json::json!({
                        "error": "Failed to retrieve credential details"
                    })),
                }
            }
        },
        _ => IntegrationStatus {
            name: "slack".to_string(),
            enabled: false,
            configured: false,
            valid: false,
            details: Some(serde_json::json!({
                "error": "No active Slack credentials found"
            })),
        }
    };
    integrations.push(slack_status);

    // Linear Integration - check for stored credentials
    let linear_status = match service.credential_service.has_active_credentials_for_type(IntegrationType::Linear).await {
        Ok(true) => {
            match service.credential_service.get_first_active_credential_for_type(IntegrationType::Linear).await {
                Ok(Some(credential)) => IntegrationStatus {
                    name: "linear".to_string(),
                    enabled: true,
                    configured: true,
                    valid: true,
                    details: Some(serde_json::json!({
                        "credential_name": credential.name,
                        "credential_id": credential.id,
                        "last_used": credential.last_used_at
                    })),
                },
                _ => IntegrationStatus {
                    name: "linear".to_string(),
                    enabled: false,
                    configured: false,
                    valid: false,
                    details: Some(serde_json::json!({
                        "error": "Failed to retrieve credential details"
                    })),
                }
            }
        },
        _ => IntegrationStatus {
            name: "linear".to_string(),
            enabled: false,
            configured: false,
            valid: false,
            details: Some(serde_json::json!({
                "error": "No active Linear credentials found"
            })),
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
    let status = match integration_name.as_str() {
        "github" => {
            match service.credential_service.get_first_credential_data_for_type(IntegrationType::GitHub).await {
                Ok(Some(CredentialData::GitHub { access_token, default_repo, webhook_secret })) => {
                    // TODO: Actually test GitHub API connection using the access_token
                    IntegrationStatus {
                        name: "github".to_string(),
                        enabled: true,
                        configured: true,
                        valid: !access_token.is_empty(),
                        details: Some(serde_json::json!({
                            "default_repo": default_repo,
                            "has_webhook_secret": webhook_secret.is_some(),
                            "test_result": "Connection test not implemented yet"
                        })),
                    }
                },
                Ok(Some(_)) => {
                    IntegrationStatus {
                        name: "github".to_string(),
                        enabled: false,
                        configured: false,
                        valid: false,
                        details: Some(serde_json::json!({
                            "error": "Invalid credential type for GitHub integration"
                        })),
                    }
                },
                Ok(None) => {
                    IntegrationStatus {
                        name: "github".to_string(),
                        enabled: false,
                        configured: false,
                        valid: false,
                        details: Some(serde_json::json!({
                            "error": "GitHub integration not configured"
                        })),
                    }
                },
                Err(e) => {
                    IntegrationStatus {
                        name: "github".to_string(),
                        enabled: false,
                        configured: false,
                        valid: false,
                        details: Some(serde_json::json!({
                            "error": format!("Failed to retrieve GitHub credentials: {}", e)
                        })),
                    }
                }
            }
        }
        "slack" => {
            match service.credential_service.get_first_credential_data_for_type(IntegrationType::Slack).await {
                Ok(Some(CredentialData::Slack { bot_token, user_token, workspace })) => {
                    // TODO: Actually test Slack API connection using the bot_token
                    IntegrationStatus {
                        name: "slack".to_string(),
                        enabled: true,
                        configured: true,
                        valid: !bot_token.is_empty(),
                        details: Some(serde_json::json!({
                            "workspace": workspace,
                            "has_user_token": user_token.is_some(),
                            "test_result": "Connection test not implemented yet"
                        })),
                    }
                },
                Ok(Some(_)) => {
                    IntegrationStatus {
                        name: "slack".to_string(),
                        enabled: false,
                        configured: false,
                        valid: false,
                        details: Some(serde_json::json!({
                            "error": "Invalid credential type for Slack integration"
                        })),
                    }
                },
                Ok(None) => {
                    IntegrationStatus {
                        name: "slack".to_string(),
                        enabled: false,
                        configured: false,
                        valid: false,
                        details: Some(serde_json::json!({
                            "error": "Slack integration not configured"
                        })),
                    }
                },
                Err(e) => {
                    IntegrationStatus {
                        name: "slack".to_string(),
                        enabled: false,
                        configured: false,
                        valid: false,
                        details: Some(serde_json::json!({
                            "error": format!("Failed to retrieve Slack credentials: {}", e)
                        })),
                    }
                }
            }
        }
        "linear" => {
            match service.credential_service.get_first_credential_data_for_type(IntegrationType::Linear).await {
                Ok(Some(CredentialData::Linear { api_key, workspace })) => {
                    // TODO: Actually test Linear API connection using the api_key
                    IntegrationStatus {
                        name: "linear".to_string(),
                        enabled: true,
                        configured: true,
                        valid: !api_key.is_empty(),
                        details: Some(serde_json::json!({
                            "workspace": workspace,
                            "test_result": "Connection test not implemented yet"
                        })),
                    }
                },
                Ok(Some(_)) => {
                    IntegrationStatus {
                        name: "linear".to_string(),
                        enabled: false,
                        configured: false,
                        valid: false,
                        details: Some(serde_json::json!({
                            "error": "Invalid credential type for Linear integration"
                        })),
                    }
                },
                Ok(None) => {
                    IntegrationStatus {
                        name: "linear".to_string(),
                        enabled: false,
                        configured: false,
                        valid: false,
                        details: Some(serde_json::json!({
                            "error": "Linear integration not configured"
                        })),
                    }
                },
                Err(e) => {
                    IntegrationStatus {
                        name: "linear".to_string(),
                        enabled: false,
                        configured: false,
                        valid: false,
                        details: Some(serde_json::json!({
                            "error": format!("Failed to retrieve Linear credentials: {}", e)
                        })),
                    }
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
    let github_data = service.credential_service
        .get_first_credential_data_for_type(IntegrationType::GitHub)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to retrieve GitHub credentials: {}", e)
            })),
        ))?
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "GitHub integration not configured"
            })),
        ))?;

    let (access_token, default_repo) = match github_data {
        CredentialData::GitHub { access_token, default_repo, .. } => {
            if access_token.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "GitHub access token is empty"
                    })),
                ));
            }
            (access_token, default_repo.unwrap_or_else(|| "N/A".to_string()))
        },
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid credential type for GitHub integration"
                })),
            ));
        }
    };

    // TODO: Implement actual webhook creation with GitHub API using access_token
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

    println!("Created GitHub webhook for repository: {}", default_repo);

    Ok((StatusCode::CREATED, Json(webhook_response)))
}

pub async fn sync_github_repository(
    Extension(service): Extension<Arc<TaskService>>,
    Json(request): Json<GitHubSyncRequest>,
) -> Result<Json<GitHubSyncResponse>, (StatusCode, Json<serde_json::Value>)> {
    let github_data = service.credential_service
        .get_first_credential_data_for_type(IntegrationType::GitHub)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": format!("Failed to retrieve GitHub credentials: {}", e)
            })),
        ))?
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "GitHub integration not configured"
            })),
        ))?;

    let (access_token, default_repo) = match github_data {
        CredentialData::GitHub { access_token, default_repo, .. } => {
            if access_token.is_empty() {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "error": "GitHub access token is empty"
                    })),
                ));
            }
            (access_token, default_repo)
        },
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid credential type for GitHub integration"
                })),
            ));
        }
    };

    let repository = request.repository
        .or(default_repo)
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "No repository specified and no default repository configured"
            })),
        ))?;
    
    let include_issues = request.include_issues.unwrap_or(true);
    let include_prs = request.include_prs.unwrap_or(true);

    // TODO: Implement actual GitHub synchronization using access_token
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
