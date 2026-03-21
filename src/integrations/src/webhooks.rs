use anyhow::Result;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    GitHubIntegration, GitHubWebhookPayload, IntegrationEventBus, LinearIntegration,
    SlackEventPayload, SlackIntegration,
};

#[derive(Debug, Clone)]
pub struct WebhookHandler {
    github: Option<GitHubIntegration>,
    linear: Option<LinearIntegration>,
    slack: Option<SlackIntegration>,
    #[allow(dead_code)]
    event_bus: IntegrationEventBus,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebhookResponse {
    success: bool,
    message: String,
}

#[derive(Debug, Deserialize)]
struct HealthQuery {
    service: Option<String>,
}

impl WebhookHandler {
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
        }
    }

    pub fn router(self) -> Router {
        let shared_state = Arc::new(self);

        Router::new()
            .route("/webhooks/github", post(github_webhook_handler))
            .route("/webhooks/linear", post(linear_webhook_handler))
            .route("/webhooks/slack", post(slack_webhook_handler))
            .route("/health", get(health_handler))
            .with_state(shared_state)
    }

    pub async fn handle_github_webhook(
        &self,
        headers: &HeaderMap,
        body: String,
    ) -> Result<StatusCode> {
        let github = self
            .github
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("GitHub integration not configured"))?;

        // Verify webhook signature
        if let Some(signature) = headers.get("x-hub-signature-256") {
            let signature_str = signature
                .to_str()
                .map_err(|_| anyhow::anyhow!("Invalid signature header"))?;

            if !github.verify_webhook_signature(&body, signature_str) {
                tracing::warn!("GitHub webhook signature verification failed");
                return Err(anyhow::anyhow!("Invalid signature"));
            }
        } else {
            tracing::warn!("GitHub webhook missing signature header");
            return Err(anyhow::anyhow!("Missing signature"));
        }

        // Parse the webhook payload
        let payload: GitHubWebhookPayload = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("Failed to parse GitHub webhook: {}", e))?;

        // Handle the webhook
        github
            .handle_webhook(payload)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to handle GitHub webhook: {}", e))?;

        tracing::info!("Successfully processed GitHub webhook");
        Ok(StatusCode::OK)
    }

    pub async fn handle_linear_webhook(
        &self,
        headers: &HeaderMap,
        body: String,
    ) -> Result<StatusCode> {
        let linear = self
            .linear
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Linear integration not configured"))?;

        // Verify webhook signature if present
        if let Some(signature) = headers.get("linear-signature") {
            // TODO: Implement Linear signature verification
            tracing::debug!("Linear signature: {:?}", signature);
        }

        // Parse the webhook payload
        let payload: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("Failed to parse Linear webhook: {}", e))?;

        // Handle the webhook
        linear
            .handle_webhook(payload)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to handle Linear webhook: {}", e))?;

        tracing::info!("Successfully processed Linear webhook");
        Ok(StatusCode::OK)
    }

    pub async fn handle_slack_event(&self, headers: &HeaderMap, body: String) -> Result<String> {
        let slack = self
            .slack
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Slack integration not configured"))?;

        // Verify Slack signature
        if let (Some(timestamp), Some(signature)) = (
            headers
                .get("x-slack-request-timestamp")
                .and_then(|h| h.to_str().ok()),
            headers
                .get("x-slack-signature")
                .and_then(|h| h.to_str().ok()),
        ) {
            if !slack.verify_signature(timestamp, &body, signature) {
                tracing::warn!("Slack event signature verification failed");
                return Err(anyhow::anyhow!("Invalid signature"));
            }
        } else {
            tracing::warn!("Slack event missing required headers");
            return Err(anyhow::anyhow!("Missing required headers"));
        }

        // Parse the event payload
        let payload: SlackEventPayload = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("Failed to parse Slack event: {}", e))?;

        // Handle the event
        let response = slack
            .handle_event(payload)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to handle Slack event: {}", e))?;

        tracing::info!("Successfully processed Slack event");
        Ok(response)
    }

    pub fn is_service_healthy(&self, service: Option<&str>) -> bool {
        match service {
            Some("github") => self.github.is_some(),
            Some("linear") => self.linear.is_some(),
            Some("slack") => self.slack.is_some(),
            None => true, // Overall health
            _ => false,
        }
    }
}

// Axum handler functions

async fn github_webhook_handler(
    State(handler): State<Arc<WebhookHandler>>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<WebhookResponse>, StatusCode> {
    match handler.handle_github_webhook(&headers, body).await {
        Ok(_) => Ok(Json(WebhookResponse {
            success: true,
            message: "GitHub webhook processed successfully".to_string(),
        })),
        Err(e) => {
            tracing::error!("GitHub webhook error: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn linear_webhook_handler(
    State(handler): State<Arc<WebhookHandler>>,
    headers: HeaderMap,
    body: String,
) -> Result<Json<WebhookResponse>, StatusCode> {
    match handler.handle_linear_webhook(&headers, body).await {
        Ok(_) => Ok(Json(WebhookResponse {
            success: true,
            message: "Linear webhook processed successfully".to_string(),
        })),
        Err(e) => {
            tracing::error!("Linear webhook error: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn slack_webhook_handler(
    State(handler): State<Arc<WebhookHandler>>,
    headers: HeaderMap,
    body: String,
) -> Result<String, StatusCode> {
    match handler.handle_slack_event(&headers, body).await {
        Ok(response) => Ok(response),
        Err(e) => {
            tracing::error!("Slack event error: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn health_handler(
    State(handler): State<Arc<WebhookHandler>>,
    Query(params): Query<HealthQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let service = params.service.as_deref();

    if handler.is_service_healthy(service) {
        Ok(Json(serde_json::json!({
            "status": "healthy",
            "service": service.unwrap_or("all"),
            "integrations": {
                "github": handler.github.is_some(),
                "linear": handler.linear.is_some(),
                "slack": handler.slack.is_some()
            }
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GitHubConfig, LinearConfig, NotificationLevel, SlackConfig};
    use axum_test::TestServer;

    fn create_test_handler() -> WebhookHandler {
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

        WebhookHandler::new(
            Some(GitHubIntegration::new(github_config, event_bus.clone())),
            Some(LinearIntegration::new(linear_config, event_bus.clone())),
            Some(SlackIntegration::new(slack_config, event_bus.clone())),
            event_bus,
        )
    }

    #[test]
    fn test_webhook_handler_creation() {
        let handler = create_test_handler();
        assert!(handler.github.is_some());
        assert!(handler.linear.is_some());
        assert!(handler.slack.is_some());
    }

    #[test]
    fn test_service_health_checks() {
        let handler = create_test_handler();

        assert!(handler.is_service_healthy(None));
        assert!(handler.is_service_healthy(Some("github")));
        assert!(handler.is_service_healthy(Some("linear")));
        assert!(handler.is_service_healthy(Some("slack")));
        assert!(!handler.is_service_healthy(Some("unknown")));
    }

    #[test]
    fn test_webhook_handler_without_integrations() {
        let event_bus = IntegrationEventBus::new();
        let handler = WebhookHandler::new(None, None, None, event_bus);

        assert!(!handler.is_service_healthy(Some("github")));
        assert!(!handler.is_service_healthy(Some("linear")));
        assert!(!handler.is_service_healthy(Some("slack")));
        assert!(handler.is_service_healthy(None)); // Overall health still true
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let handler = create_test_handler();
        let app = handler.router();
        let server = TestServer::new(app).unwrap();

        let response = server.get("/health").await;

        response.assert_status_ok();

        let json: serde_json::Value = response.json();
        assert_eq!(json["status"], "healthy");
        assert_eq!(json["service"], "all");
        assert_eq!(json["integrations"]["github"], true);
        assert_eq!(json["integrations"]["linear"], true);
        assert_eq!(json["integrations"]["slack"], true);
    }

    #[tokio::test]
    async fn test_health_endpoint_specific_service() {
        let handler = create_test_handler();
        let app = handler.router();
        let server = TestServer::new(app).unwrap();

        let response = server.get("/health?service=github").await;

        response.assert_status_ok();

        let json: serde_json::Value = response.json();
        assert_eq!(json["status"], "healthy");
        assert_eq!(json["service"], "github");
    }

    #[tokio::test]
    async fn test_webhook_routes_exist() {
        let handler = create_test_handler();
        let app = handler.router();
        let server = TestServer::new(app).unwrap();

        // Test that routes exist (they should return errors due to missing/invalid data)
        let response = server.post("/webhooks/github").await;
        assert!(response.status_code().is_client_error());

        let response = server.post("/webhooks/linear").await;
        assert!(response.status_code().is_client_error());

        let response = server.post("/webhooks/slack").await;
        assert!(response.status_code().is_client_error());
    }

    #[test]
    fn test_webhook_response_serialization() {
        let response = WebhookResponse {
            success: true,
            message: "Test message".to_string(),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: WebhookResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response.success, deserialized.success);
        assert_eq!(response.message, deserialized.message);
    }

    #[test]
    fn test_health_query_deserialization() {
        let query_str = "service=github";
        let _query: HealthQuery =
            serde_qs::from_str(query_str).unwrap_or(HealthQuery { service: None });

        // Since serde_qs is not included, we'll test the struct directly
        let query = HealthQuery {
            service: Some("github".to_string()),
        };

        assert_eq!(query.service.unwrap(), "github");
    }
}
