use crate::{IntegrationEventBus, TaskChanges};
use anyhow::Result;
use luce_shared::{GitHubAttachment, GitHubPRState, TaskAttachment, TaskId};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String, // "open", "closed"
    pub merged: bool,
    pub draft: bool,
    pub user: GitHubUser,
    pub base: GitHubPRBranch,
    pub head: GitHubPRBranch,
    pub html_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPRBranch {
    pub ref_name: String,
    pub repo: GitHubRepository,
}

#[derive(Debug, Serialize)]
struct CreatePRRequest {
    title: String,
    body: Option<String>,
    head: String,
    base: String,
    draft: Option<bool>,
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

    // Pull Request Methods

    pub async fn create_pull_request(
        &self,
        _task_id: TaskId,
        title: &str,
        body: Option<&str>,
        head_branch: &str,
        base_branch: &str,
        draft: bool,
    ) -> Result<GitHubPullRequest> {
        let url = format!(
            "https://api.github.com/repos/{}/pulls",
            self.config.default_repo
        );

        let request_body = CreatePRRequest {
            title: title.to_string(),
            body: body.map(|s| s.to_string()),
            head: head_branch.to_string(),
            base: base_branch.to_string(),
            draft: Some(draft),
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

        let pr: GitHubPullRequest = response.json().await?;
        Ok(pr)
    }

    pub async fn get_pull_request(&self, pr_number: u64) -> Result<GitHubPullRequest> {
        let url = format!(
            "https://api.github.com/repos/{}/pulls/{}",
            self.config.default_repo, pr_number
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

        let pr: GitHubPullRequest = response.json().await?;
        Ok(pr)
    }

    /// Create a task attachment from a GitHub pull request
    pub fn create_pr_attachment(&self, task_id: TaskId, pr: &GitHubPullRequest) -> TaskAttachment {
        let state = if pr.merged {
            GitHubPRState::Merged
        } else if pr.draft {
            GitHubPRState::Draft
        } else if pr.state == "closed" {
            GitHubPRState::Closed
        } else {
            GitHubPRState::Open
        };

        let github_attachment = GitHubAttachment {
            repository: self.config.default_repo.clone(),
            pr_number: pr.number,
            title: pr.title.clone(),
            state,
            author: pr.user.login.clone(),
            base_branch: pr.base.ref_name.clone(),
            head_branch: pr.head.ref_name.clone(),
            url: pr.html_url.clone(),
        };

        TaskAttachment::new_github(task_id, github_attachment)
    }

    /// Attach a GitHub pull request to a task
    pub async fn attach_pull_request(
        &self,
        task_id: TaskId,
        pr_number: u64,
    ) -> Result<TaskAttachment> {
        let pr = self.get_pull_request(pr_number).await?;
        let attachment = self.create_pr_attachment(task_id, &pr);
        Ok(attachment)
    }

    /// Create a pull request and immediately attach it to a task
    pub async fn create_and_attach_pull_request(
        &self,
        task_id: TaskId,
        title: &str,
        body: Option<&str>,
        head_branch: &str,
        base_branch: &str,
        draft: bool,
    ) -> Result<TaskAttachment> {
        let pr = self
            .create_pull_request(task_id, title, body, head_branch, base_branch, draft)
            .await?;
        let attachment = self.create_pr_attachment(task_id, &pr);
        Ok(attachment)
    }

    /// Update an attachment with the latest PR data
    pub async fn refresh_pr_attachment(&self, attachment: &mut TaskAttachment) -> Result<()> {
        match &attachment.data {
            luce_shared::AttachmentData::GitHub(github_data) => {
                let pr = self.get_pull_request(github_data.pr_number).await?;

                // Create a new attachment with updated data and merge it
                let updated_attachment = self.create_pr_attachment(attachment.task_id, &pr);
                if let luce_shared::AttachmentData::GitHub(updated_github_data) =
                    updated_attachment.data
                {
                    attachment.data = luce_shared::AttachmentData::GitHub(updated_github_data);
                    attachment.touch();
                }
            }
            _ => return Err(anyhow::anyhow!("Attachment is not a GitHub attachment")),
        }
        Ok(())
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

    #[test]
    fn test_create_pr_attachment() {
        let event_bus = IntegrationEventBus::new();
        let config = GitHubConfig {
            access_token: "test_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            default_repo: "owner/repo".to_string(),
        };
        let integration = GitHubIntegration::new(config, event_bus);
        let task_id = TaskId::new_v4();

        let pr = GitHubPullRequest {
            id: 123,
            number: 42,
            title: "Add new feature".to_string(),
            body: Some("This PR adds a new feature".to_string()),
            state: "open".to_string(),
            merged: false,
            draft: false,
            user: GitHubUser {
                login: "developer".to_string(),
                id: 456,
            },
            base: GitHubPRBranch {
                ref_name: "main".to_string(),
                repo: GitHubRepository {
                    id: 789,
                    name: "repo".to_string(),
                    full_name: "owner/repo".to_string(),
                },
            },
            head: GitHubPRBranch {
                ref_name: "feature/new-feature".to_string(),
                repo: GitHubRepository {
                    id: 789,
                    name: "repo".to_string(),
                    full_name: "owner/repo".to_string(),
                },
            },
            html_url: "https://github.com/owner/repo/pull/42".to_string(),
        };

        let attachment = integration.create_pr_attachment(task_id, &pr);

        assert_eq!(attachment.task_id, task_id);
        assert_eq!(attachment.title(), "Add new feature");
        assert_eq!(attachment.url(), "https://github.com/owner/repo/pull/42");
        assert_eq!(attachment.identifier(), "owner/repo#42");

        match &attachment.data {
            luce_shared::AttachmentData::GitHub(github_data) => {
                assert_eq!(github_data.repository, "owner/repo");
                assert_eq!(github_data.pr_number, 42);
                assert_eq!(github_data.title, "Add new feature");
                assert_eq!(github_data.state, luce_shared::GitHubPRState::Open);
                assert_eq!(github_data.author, "developer");
                assert_eq!(github_data.base_branch, "main");
                assert_eq!(github_data.head_branch, "feature/new-feature");
            }
            _ => panic!("Expected GitHub attachment"),
        }
    }

    #[test]
    fn test_pr_state_mapping() {
        let event_bus = IntegrationEventBus::new();
        let config = GitHubConfig {
            access_token: "test_token".to_string(),
            webhook_secret: "test_secret".to_string(),
            default_repo: "owner/repo".to_string(),
        };
        let integration = GitHubIntegration::new(config, event_bus);
        let task_id = TaskId::new_v4();

        // Test different PR states
        let test_cases = vec![
            (true, false, "closed", luce_shared::GitHubPRState::Merged), // merged
            (false, true, "open", luce_shared::GitHubPRState::Draft),    // draft
            (false, false, "closed", luce_shared::GitHubPRState::Closed), // closed
            (false, false, "open", luce_shared::GitHubPRState::Open),    // open
        ];

        for (merged, draft, state, expected_state) in test_cases {
            let pr = GitHubPullRequest {
                id: 123,
                number: 42,
                title: "Test PR".to_string(),
                body: None,
                state: state.to_string(),
                merged,
                draft,
                user: GitHubUser {
                    login: "dev".to_string(),
                    id: 456,
                },
                base: GitHubPRBranch {
                    ref_name: "main".to_string(),
                    repo: GitHubRepository {
                        id: 789,
                        name: "repo".to_string(),
                        full_name: "owner/repo".to_string(),
                    },
                },
                head: GitHubPRBranch {
                    ref_name: "feature".to_string(),
                    repo: GitHubRepository {
                        id: 789,
                        name: "repo".to_string(),
                        full_name: "owner/repo".to_string(),
                    },
                },
                html_url: "https://github.com/owner/repo/pull/42".to_string(),
            };

            let attachment = integration.create_pr_attachment(task_id, &pr);

            if let luce_shared::AttachmentData::GitHub(github_data) = &attachment.data {
                assert_eq!(github_data.state, expected_state);
            } else {
                panic!("Expected GitHub attachment");
            }
        }
    }
}
