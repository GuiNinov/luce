use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

pub type CredentialId = Uuid;
pub type IntegrationId = Uuid;
pub type AttachmentId = Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrationType {
    GitHub,
    Slack,
    Linear,
}

impl std::fmt::Display for IntegrationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntegrationType::GitHub => write!(f, "github"),
            IntegrationType::Slack => write!(f, "slack"),
            IntegrationType::Linear => write!(f, "linear"),
        }
    }
}

impl std::str::FromStr for IntegrationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "github" => Ok(IntegrationType::GitHub),
            "slack" => Ok(IntegrationType::Slack),
            "linear" => Ok(IntegrationType::Linear),
            _ => Err(format!("Unknown integration type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationCredential {
    pub id: CredentialId,
    pub integration_type: IntegrationType,
    pub name: String,
    pub encrypted_data: String, // Encrypted JSON containing actual credentials
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateCredentialInput {
    pub integration_type: IntegrationType,
    pub name: String,
    pub credentials: CredentialData, // This will be encrypted when stored
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateCredentialInput {
    pub name: Option<String>,
    pub credentials: Option<CredentialData>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CredentialData {
    GitHub {
        access_token: String,
        default_repo: Option<String>,
        webhook_secret: Option<String>,
    },
    Slack {
        bot_token: String,
        user_token: Option<String>,
        workspace: String,
    },
    Linear {
        api_key: String,
        workspace: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    Never,
    Success,
    Error,
    InProgress,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncStatus::Never => write!(f, "never"),
            SyncStatus::Success => write!(f, "success"),
            SyncStatus::Error => write!(f, "error"),
            SyncStatus::InProgress => write!(f, "in_progress"),
        }
    }
}

impl std::str::FromStr for SyncStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "never" => Ok(SyncStatus::Never),
            "success" => Ok(SyncStatus::Success),
            "error" => Ok(SyncStatus::Error),
            "in_progress" => Ok(SyncStatus::InProgress),
            _ => Err(format!("Unknown sync status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Integration {
    pub id: IntegrationId,
    pub integration_type: IntegrationType,
    pub credential_id: Option<CredentialId>,
    pub config_data: HashMap<String, serde_json::Value>, // JSON config
    pub is_enabled: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub sync_status: SyncStatus,
    pub sync_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateIntegrationInput {
    pub integration_type: IntegrationType,
    pub credential_id: Option<CredentialId>,
    pub config_data: HashMap<String, serde_json::Value>,
    pub is_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateIntegrationInput {
    pub credential_id: Option<CredentialId>,
    pub config_data: Option<HashMap<String, serde_json::Value>>,
    pub is_enabled: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttachmentType {
    Issue,
    PullRequest,
    Thread,
    Ticket,
    Document,
}

impl std::fmt::Display for AttachmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttachmentType::Issue => write!(f, "issue"),
            AttachmentType::PullRequest => write!(f, "pr"),
            AttachmentType::Thread => write!(f, "thread"),
            AttachmentType::Ticket => write!(f, "ticket"),
            AttachmentType::Document => write!(f, "document"),
        }
    }
}

impl std::str::FromStr for AttachmentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "issue" => Ok(AttachmentType::Issue),
            "pr" | "pull_request" | "pullrequest" => Ok(AttachmentType::PullRequest),
            "thread" => Ok(AttachmentType::Thread),
            "ticket" => Ok(AttachmentType::Ticket),
            "document" => Ok(AttachmentType::Document),
            _ => Err(format!("Unknown attachment type: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttachmentStatus {
    Active,
    Deleted,
    Archived,
}

impl std::fmt::Display for AttachmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttachmentStatus::Active => write!(f, "active"),
            AttachmentStatus::Deleted => write!(f, "deleted"),
            AttachmentStatus::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for AttachmentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(AttachmentStatus::Active),
            "deleted" => Ok(AttachmentStatus::Deleted),
            "archived" => Ok(AttachmentStatus::Archived),
            _ => Err(format!("Unknown attachment status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskAttachment {
    pub id: AttachmentId,
    pub task_id: crate::TaskId,
    pub integration_type: IntegrationType,
    pub external_id: String,
    pub external_url: Option<String>,
    pub attachment_data: HashMap<String, serde_json::Value>, // JSON data
    pub attachment_type: AttachmentType,
    pub status: AttachmentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateAttachmentInput {
    pub task_id: crate::TaskId,
    pub integration_type: IntegrationType,
    pub external_id: String,
    pub external_url: Option<String>,
    pub attachment_data: HashMap<String, serde_json::Value>,
    pub attachment_type: AttachmentType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateAttachmentInput {
    pub external_url: Option<String>,
    pub attachment_data: Option<HashMap<String, serde_json::Value>>,
    pub status: Option<AttachmentStatus>,
}

impl IntegrationCredential {
    pub fn new(
        integration_type: IntegrationType,
        name: String,
        encrypted_data: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            integration_type,
            name,
            encrypted_data,
            is_active: true,
            created_at: now,
            updated_at: now,
            last_used_at: None,
        }
    }

    pub fn mark_used(&mut self) {
        self.last_used_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

impl Integration {
    pub fn new(
        integration_type: IntegrationType,
        credential_id: Option<CredentialId>,
        config_data: HashMap<String, serde_json::Value>,
        is_enabled: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            integration_type,
            credential_id,
            config_data,
            is_enabled,
            last_sync_at: None,
            sync_status: SyncStatus::Never,
            sync_error: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn start_sync(&mut self) {
        self.sync_status = SyncStatus::InProgress;
        self.sync_error = None;
        self.updated_at = Utc::now();
    }

    pub fn complete_sync(&mut self, success: bool, error: Option<String>) {
        self.sync_status = if success { SyncStatus::Success } else { SyncStatus::Error };
        self.sync_error = error;
        self.last_sync_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

impl TaskAttachment {
    pub fn new(
        task_id: crate::TaskId,
        integration_type: IntegrationType,
        external_id: String,
        external_url: Option<String>,
        attachment_data: HashMap<String, serde_json::Value>,
        attachment_type: AttachmentType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            task_id,
            integration_type,
            external_id,
            external_url,
            attachment_data,
            attachment_type,
            status: AttachmentStatus::Active,
            created_at: now,
            updated_at: now,
            synced_at: None,
        }
    }

    pub fn mark_synced(&mut self) {
        self.synced_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_type_display() {
        assert_eq!(IntegrationType::GitHub.to_string(), "github");
        assert_eq!(IntegrationType::Slack.to_string(), "slack");
        assert_eq!(IntegrationType::Linear.to_string(), "linear");
    }

    #[test]
    fn test_integration_type_from_str() {
        assert_eq!("github".parse::<IntegrationType>().unwrap(), IntegrationType::GitHub);
        assert_eq!("GITHUB".parse::<IntegrationType>().unwrap(), IntegrationType::GitHub);
        assert!("unknown".parse::<IntegrationType>().is_err());
    }

    #[test]
    fn test_credential_creation() {
        let cred = IntegrationCredential::new(
            IntegrationType::GitHub,
            "Work Account".to_string(),
            "encrypted_data".to_string(),
        );
        
        assert_eq!(cred.integration_type, IntegrationType::GitHub);
        assert_eq!(cred.name, "Work Account");
        assert!(cred.is_active);
        assert!(cred.last_used_at.is_none());
    }

    #[test]
    fn test_integration_sync_workflow() {
        let mut integration = Integration::new(
            IntegrationType::GitHub,
            Some(Uuid::new_v4()),
            HashMap::new(),
            true,
        );

        assert_eq!(integration.sync_status, SyncStatus::Never);
        
        integration.start_sync();
        assert_eq!(integration.sync_status, SyncStatus::InProgress);
        assert!(integration.sync_error.is_none());

        integration.complete_sync(true, None);
        assert_eq!(integration.sync_status, SyncStatus::Success);
        assert!(integration.last_sync_at.is_some());

        integration.start_sync();
        integration.complete_sync(false, Some("Error message".to_string()));
        assert_eq!(integration.sync_status, SyncStatus::Error);
        assert_eq!(integration.sync_error, Some("Error message".to_string()));
    }

    #[test]
    fn test_attachment_creation() {
        let attachment = TaskAttachment::new(
            Uuid::new_v4(),
            IntegrationType::GitHub,
            "123".to_string(),
            Some("https://github.com/owner/repo/issues/123".to_string()),
            HashMap::new(),
            AttachmentType::Issue,
        );

        assert_eq!(attachment.integration_type, IntegrationType::GitHub);
        assert_eq!(attachment.external_id, "123");
        assert_eq!(attachment.attachment_type, AttachmentType::Issue);
        assert_eq!(attachment.status, AttachmentStatus::Active);
        assert!(attachment.synced_at.is_none());
    }

    #[test]
    fn test_credential_data_serialization() {
        let github_creds = CredentialData::GitHub {
            access_token: "token123".to_string(),
            default_repo: Some("owner/repo".to_string()),
            webhook_secret: Some("secret".to_string()),
        };

        let serialized = serde_json::to_string(&github_creds).unwrap();
        let deserialized: CredentialData = serde_json::from_str(&serialized).unwrap();

        assert_eq!(github_creds, deserialized);
    }
}