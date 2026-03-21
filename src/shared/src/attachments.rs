use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::TaskId;

pub type AttachmentId = Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAttachment {
    pub id: AttachmentId,
    pub task_id: TaskId,
    pub data: AttachmentData,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttachmentData {
    GitHub(GitHubAttachment),
    Slack(SlackAttachment),
    Linear(LinearAttachment),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitHubAttachment {
    pub repository: String,
    pub pr_number: u64,
    pub title: String,
    pub state: GitHubPRState,
    pub author: String,
    pub base_branch: String,
    pub head_branch: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GitHubPRState {
    Open,
    Draft,
    Closed,
    Merged,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlackAttachment {
    pub channel: String,
    pub thread_ts: String,
    pub message_ts: String,
    pub url: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearAttachment {
    pub issue_id: String,
    pub title: String,
    pub state: LinearIssueState,
    pub assignee: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LinearIssueState {
    Backlog,
    Todo,
    InProgress,
    InReview,
    Done,
    Cancelled,
}

impl TaskAttachment {
    pub fn new_github(task_id: TaskId, github_data: GitHubAttachment) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            task_id,
            data: AttachmentData::GitHub(github_data),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn new_slack(task_id: TaskId, slack_data: SlackAttachment) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            task_id,
            data: AttachmentData::Slack(slack_data),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn new_linear(task_id: TaskId, linear_data: LinearAttachment) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            task_id,
            data: AttachmentData::Linear(linear_data),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    pub fn title(&self) -> &str {
        match &self.data {
            AttachmentData::GitHub(github) => &github.title,
            AttachmentData::Slack(slack) => &slack.title,
            AttachmentData::Linear(linear) => &linear.title,
        }
    }

    pub fn url(&self) -> &str {
        match &self.data {
            AttachmentData::GitHub(github) => &github.url,
            AttachmentData::Slack(slack) => &slack.url,
            AttachmentData::Linear(linear) => &linear.url,
        }
    }

    pub fn identifier(&self) -> String {
        match &self.data {
            AttachmentData::GitHub(github) => format!("{}#{}", github.repository, github.pr_number),
            AttachmentData::Slack(slack) => format!("#{}", slack.channel),
            AttachmentData::Linear(linear) => linear.issue_id.clone(),
        }
    }

    pub fn attachment_type(&self) -> &'static str {
        match &self.data {
            AttachmentData::GitHub(_) => "github",
            AttachmentData::Slack(_) => "slack",
            AttachmentData::Linear(_) => "linear",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_github_attachment() {
        let task_id = Uuid::new_v4();
        let github_data = GitHubAttachment {
            repository: "owner/repo".to_string(),
            pr_number: 42,
            title: "Add new feature".to_string(),
            state: GitHubPRState::Open,
            author: "developer".to_string(),
            base_branch: "main".to_string(),
            head_branch: "feature/new-feature".to_string(),
            url: "https://github.com/owner/repo/pull/42".to_string(),
        };

        let attachment = TaskAttachment::new_github(task_id, github_data.clone());

        assert_eq!(attachment.task_id, task_id);
        assert_eq!(attachment.title(), "Add new feature");
        assert_eq!(attachment.url(), "https://github.com/owner/repo/pull/42");
        assert_eq!(attachment.identifier(), "owner/repo#42");
        assert_eq!(attachment.attachment_type(), "github");

        if let AttachmentData::GitHub(github) = &attachment.data {
            assert_eq!(github.repository, "owner/repo");
            assert_eq!(github.pr_number, 42);
            assert_eq!(github.state, GitHubPRState::Open);
        } else {
            panic!("Expected GitHub attachment");
        }
    }

    #[test]
    fn test_create_slack_attachment() {
        let task_id = Uuid::new_v4();
        let slack_data = SlackAttachment {
            channel: "general".to_string(),
            thread_ts: "1234567890.123456".to_string(),
            message_ts: "1234567890.123456".to_string(),
            url: "https://workspace.slack.com/archives/C1234567890/p1234567890123456".to_string(),
            title: "Task discussion".to_string(),
        };

        let attachment = TaskAttachment::new_slack(task_id, slack_data.clone());

        assert_eq!(attachment.task_id, task_id);
        assert_eq!(attachment.title(), "Task discussion");
        assert_eq!(attachment.identifier(), "#general");
        assert_eq!(attachment.attachment_type(), "slack");
    }

    #[test]
    fn test_create_linear_attachment() {
        let task_id = Uuid::new_v4();
        let linear_data = LinearAttachment {
            issue_id: "LIN-123".to_string(),
            title: "Fix bug in authentication".to_string(),
            state: LinearIssueState::InProgress,
            assignee: Some("developer".to_string()),
            url: "https://linear.app/team/issue/LIN-123".to_string(),
        };

        let attachment = TaskAttachment::new_linear(task_id, linear_data.clone());

        assert_eq!(attachment.task_id, task_id);
        assert_eq!(attachment.title(), "Fix bug in authentication");
        assert_eq!(attachment.identifier(), "LIN-123");
        assert_eq!(attachment.attachment_type(), "linear");
    }

    #[test]
    fn test_attachment_touch_updates_timestamp() {
        let task_id = Uuid::new_v4();
        let github_data = GitHubAttachment {
            repository: "owner/repo".to_string(),
            pr_number: 42,
            title: "Test PR".to_string(),
            state: GitHubPRState::Open,
            author: "developer".to_string(),
            base_branch: "main".to_string(),
            head_branch: "feature".to_string(),
            url: "https://github.com/owner/repo/pull/42".to_string(),
        };

        let mut attachment = TaskAttachment::new_github(task_id, github_data);
        let original_updated_at = attachment.updated_at;

        // Wait a bit to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(1));
        attachment.touch();

        assert!(attachment.updated_at > original_updated_at);
    }

    #[test]
    fn test_github_pr_states() {
        let states = vec![
            GitHubPRState::Open,
            GitHubPRState::Draft,
            GitHubPRState::Closed,
            GitHubPRState::Merged,
        ];

        for state in states {
            let serialized = serde_json::to_string(&state).unwrap();
            let deserialized: GitHubPRState = serde_json::from_str(&serialized).unwrap();
            assert_eq!(state, deserialized);
        }
    }

    #[test]
    fn test_linear_issue_states() {
        let states = vec![
            LinearIssueState::Backlog,
            LinearIssueState::Todo,
            LinearIssueState::InProgress,
            LinearIssueState::InReview,
            LinearIssueState::Done,
            LinearIssueState::Cancelled,
        ];

        for state in states {
            let serialized = serde_json::to_string(&state).unwrap();
            let deserialized: LinearIssueState = serde_json::from_str(&serialized).unwrap();
            assert_eq!(state, deserialized);
        }
    }

    #[test]
    fn test_attachment_data_serialization() {
        let task_id = Uuid::new_v4();
        let github_data = GitHubAttachment {
            repository: "owner/repo".to_string(),
            pr_number: 42,
            title: "Test PR".to_string(),
            state: GitHubPRState::Open,
            author: "developer".to_string(),
            base_branch: "main".to_string(),
            head_branch: "feature".to_string(),
            url: "https://github.com/owner/repo/pull/42".to_string(),
        };

        let attachment = TaskAttachment::new_github(task_id, github_data);
        let serialized = serde_json::to_string(&attachment).unwrap();
        let deserialized: TaskAttachment = serde_json::from_str(&serialized).unwrap();

        assert_eq!(attachment, deserialized);
    }
}
