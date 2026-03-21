use crate::{IntegrationEventBus, TaskChanges};
use anyhow::Result;
use luce_shared::TaskId;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub access_token: String,
    pub webhook_secret: String,
    pub default_repo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u32,
    pub title: String,
    pub body: Option<String>,
    pub state: GitHubIssueState,
    pub assignee: Option<GitHubUser>,
    pub labels: Vec<GitHubLabel>,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GitHubIssueState {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "closed")]
    Closed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubWebhookPayload {
    pub action: String,
    pub issue: Option<GitHubIssue>,
    pub repository: GitHubRepository,
    pub sender: GitHubUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
}

#[derive(Debug, Serialize)]
struct CreateIssueRequest {
    title: String,
    body: Option<String>,
    labels: Vec<String>,
    assignees: Vec<String>,
}

#[derive(Debug, Serialize)]
struct UpdateIssueRequest {
    title: Option<String>,
    body: Option<String>,
    state: Option<String>,
    labels: Option<Vec<String>>,
    assignees: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct GitHubIntegration {
    config: GitHubConfig,
    client: Client,
    #[allow(dead_code)]
    event_bus: IntegrationEventBus,
}

impl GitHubIntegration {
    pub fn new(config: GitHubConfig, event_bus: IntegrationEventBus) -> Self {
        Self {
            config,
            client: Client::new(),
            event_bus,
        }
    }

    pub async fn create_issue(
        &self,
        _task_id: TaskId,
        title: &str,
        body: Option<&str>,
    ) -> Result<GitHubIssue> {
        let url = format!(
            "https://api.github.com/repos/{}/issues",
            self.config.default_repo
        );

        let request_body = CreateIssueRequest {
            title: title.to_string(),
            body: body.map(|s| s.to_string()),
            labels: vec![],
            assignees: vec![],
        };

        let response = self
            .client
            .post(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.access_token),
            )
            .header("User-Agent", "luce-integrations")
            .json(&request_body)
            .send()
            .await?;

        let issue: GitHubIssue = response.json().await?;
        Ok(issue)
    }

    pub async fn update_issue(
        &self,
        issue_number: u32,
        updates: &TaskChanges,
    ) -> Result<GitHubIssue> {
        let url = format!(
            "https://api.github.com/repos/{}/issues/{}",
            self.config.default_repo, issue_number
        );

        let mut request_body = UpdateIssueRequest {
            title: updates.title.clone(),
            body: updates.description.clone(),
            state: None,
            labels: None,
            assignees: None,
        };

        if let Some(status) = &updates.status {
            request_body.state = Some(match status.as_str() {
                "Completed" => "closed".to_string(),
                _ => "open".to_string(),
            });
        }

        let response = self
            .client
            .patch(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.access_token),
            )
            .header("User-Agent", "luce-integrations")
            .json(&request_body)
            .send()
            .await?;

        let issue: GitHubIssue = response.json().await?;
        Ok(issue)
    }

    pub async fn get_issue(&self, issue_number: u32) -> Result<GitHubIssue> {
        let url = format!(
            "https://api.github.com/repos/{}/issues/{}",
            self.config.default_repo, issue_number
        );

        let response = self
            .client
            .get(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.config.access_token),
            )
            .header("User-Agent", "luce-integrations")
            .send()
            .await?;

        let issue: GitHubIssue = response.json().await?;
        Ok(issue)
    }

    pub async fn close_issue(&self, issue_number: u32) -> Result<GitHubIssue> {
        let updates = TaskChanges {
            status: Some("Completed".to_string()),
            title: None,
            description: None,
            priority: None,
            assignee: None,
        };

        self.update_issue(issue_number, &updates).await
    }

    pub async fn handle_webhook(&self, payload: GitHubWebhookPayload) -> Result<()> {
        tracing::info!("Handling GitHub webhook: action={}", payload.action);

        if let Some(issue) = payload.issue {
            match payload.action.as_str() {
                "opened" => {
                    tracing::info!("GitHub issue opened: #{}", issue.number);
                }
                "edited" => {
                    tracing::info!("GitHub issue edited: #{}", issue.number);
                }
                "closed" => {
                    tracing::info!("GitHub issue closed: #{}", issue.number);
                }
                _ => {
                    tracing::debug!("Unhandled GitHub action: {}", payload.action);
                }
            }
        }

        Ok(())
    }

    pub fn verify_webhook_signature(&self, payload: &str, signature: &str) -> bool {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let mut mac = match HmacSha256::new_from_slice(self.config.webhook_secret.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return false,
        };

        mac.update(payload.as_bytes());

        let expected = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

        signature == expected
    }

    pub fn task_status_to_github_state(status: &str) -> &'static str {
        match status {
            "Completed" | "Done" => "closed",
            _ => "open",
        }
    }

    pub fn github_state_to_task_status(state: &GitHubIssueState) -> &'static str {
        match state {
            GitHubIssueState::Open => "InProgress",
            GitHubIssueState::Closed => "Completed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> GitHubConfig {
        GitHubConfig {
            access_token: "test_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            default_repo: "test/repo".to_string(),
        }
    }

    #[test]
    fn test_github_integration_creation() {
        let config = create_test_config();
        let event_bus = IntegrationEventBus::new();
        let integration = GitHubIntegration::new(config.clone(), event_bus);

        assert_eq!(integration.config.default_repo, "test/repo");
    }

    #[test]
    fn test_status_conversion() {
        assert_eq!(
            GitHubIntegration::task_status_to_github_state("Completed"),
            "closed"
        );
        assert_eq!(
            GitHubIntegration::task_status_to_github_state("InProgress"),
            "open"
        );
        assert_eq!(
            GitHubIntegration::task_status_to_github_state("Ready"),
            "open"
        );

        assert_eq!(
            GitHubIntegration::github_state_to_task_status(&GitHubIssueState::Open),
            "InProgress"
        );
        assert_eq!(
            GitHubIntegration::github_state_to_task_status(&GitHubIssueState::Closed),
            "Completed"
        );
    }

    #[test]
    fn test_github_issue_serialization() {
        let issue = GitHubIssue {
            id: 123,
            number: 456,
            title: "Test Issue".to_string(),
            body: Some("Test body".to_string()),
            state: GitHubIssueState::Open,
            assignee: Some(GitHubUser {
                login: "testuser".to_string(),
                id: 789,
            }),
            labels: vec![GitHubLabel {
                name: "bug".to_string(),
                color: "ff0000".to_string(),
            }],
            html_url: "https://github.com/test/repo/issues/456".to_string(),
        };

        let serialized = serde_json::to_string(&issue).unwrap();
        let deserialized: GitHubIssue = serde_json::from_str(&serialized).unwrap();

        assert_eq!(issue.id, deserialized.id);
        assert_eq!(issue.number, deserialized.number);
        assert_eq!(issue.title, deserialized.title);
    }

    #[test]
    fn test_webhook_payload_deserialization() {
        let payload_json = r#"{
            "action": "opened",
            "issue": {
                "id": 123,
                "number": 456,
                "title": "Test Issue",
                "body": "Test body",
                "state": "open",
                "assignee": null,
                "labels": [],
                "html_url": "https://github.com/test/repo/issues/456"
            },
            "repository": {
                "id": 789,
                "name": "repo",
                "full_name": "test/repo"
            },
            "sender": {
                "login": "testuser",
                "id": 101112
            }
        }"#;

        let payload: GitHubWebhookPayload = serde_json::from_str(payload_json).unwrap();
        assert_eq!(payload.action, "opened");
        assert!(payload.issue.is_some());
        assert_eq!(payload.repository.full_name, "test/repo");
    }

    #[test]
    fn test_task_changes_to_update_request() {
        let changes = TaskChanges {
            status: Some("Completed".to_string()),
            title: Some("Updated Title".to_string()),
            description: Some("Updated description".to_string()),
            priority: Some("High".to_string()),
            assignee: Some("newuser".to_string()),
        };

        assert_eq!(changes.title.as_ref().unwrap(), "Updated Title");
        assert_eq!(changes.status.as_ref().unwrap(), "Completed");
    }
}
