pub mod events;
pub mod github;
pub mod linear;
pub mod manager;
pub mod slack;
pub mod sync;
pub mod webhooks;

use chrono::{DateTime, Utc};
use luce_shared::TaskId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntegrationType {
    GitHub,
    Linear,
    Slack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalReference {
    pub integration_type: IntegrationType,
    pub external_id: String,
    pub url: Option<String>,
    pub last_synced: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub task_id: TaskId,
    pub integration_type: IntegrationType,
    pub success: bool,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskChanges {
    pub status: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<String>,
    pub assignee: Option<String>,
}

pub use events::*;
pub use github::*;
pub use linear::*;
pub use manager::*;
pub use slack::*;
pub use sync::*;
pub use webhooks::*;
