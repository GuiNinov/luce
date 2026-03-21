use crate::{IntegrationEventBus, TaskChanges};
use anyhow::Result;
use luce_shared::TaskId;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub bot_token: String,
    pub signing_secret: String,
    pub default_channel: String,
    pub notification_levels: Vec<NotificationLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationLevel {
    StatusChanges,
    Completions,
    BlockingEvents,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackMessage {
    pub channel: String,
    pub text: String,
    pub thread_ts: Option<String>,
    pub blocks: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackUser {
    pub id: String,
    pub name: String,
    pub real_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackChannel {
    pub id: String,
    pub name: String,
    pub is_channel: bool,
    pub is_group: bool,
    pub is_im: bool,
}

#[derive(Debug, Serialize)]
struct SlackPostMessageRequest {
    channel: String,
    text: String,
    thread_ts: Option<String>,
    blocks: Option<serde_json::Value>,
    username: Option<String>,
    icon_emoji: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SlackPostMessageResponse {
    ok: bool,
    channel: Option<String>,
    ts: Option<String>,
    message: Option<SlackMessageObject>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SlackMessageObject {
    ts: String,
    text: String,
}

#[derive(Debug, Deserialize)]
pub struct SlackEventPayload {
    pub token: String,
    pub challenge: Option<String>,
    pub event: Option<SlackEvent>,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Debug, Deserialize)]
pub struct SlackEvent {
    #[serde(rename = "type")]
    pub type_: String,
    pub user: Option<String>,
    pub text: Option<String>,
    pub channel: Option<String>,
    pub ts: Option<String>,
    pub thread_ts: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SlackIntegration {
    config: SlackConfig,
    client: Client,
    #[allow(dead_code)]
    event_bus: IntegrationEventBus,
}

impl SlackIntegration {
    pub fn new(config: SlackConfig, event_bus: IntegrationEventBus) -> Self {
        Self {
            config,
            client: Client::new(),
            event_bus,
        }
    }

    pub async fn post_message(&self, message: &SlackMessage) -> Result<serde_json::Value> {
        let request_body = SlackPostMessageRequest {
            channel: message.channel.clone(),
            text: message.text.clone(),
            thread_ts: message.thread_ts.clone(),
            blocks: message.blocks.clone(),
            username: Some("Luce".to_string()),
            icon_emoji: Some(":robot_face:".to_string()),
        };

        let response = self
            .client
            .post("https://slack.com/api/chat.postMessage")
            .header("Authorization", format!("Bearer {}", self.config.bot_token))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let slack_response: SlackPostMessageResponse = response.json().await?;

        if !slack_response.ok {
            return Err(anyhow::anyhow!(
                "Slack API error: {}",
                slack_response.error.unwrap_or("Unknown error".to_string())
            ));
        }

        Ok(serde_json::to_value(slack_response)?)
    }

    pub async fn notify_task_created(&self, task_id: TaskId, title: &str) -> Result<()> {
        if !self.should_notify(&NotificationLevel::All) {
            return Ok(());
        }

        let message = SlackMessage {
            channel: self.config.default_channel.clone(),
            text: format!("🆕 New task created: *{}*\nTask ID: `{}`", title, task_id),
            thread_ts: None,
            blocks: Some(self.create_task_blocks(task_id, title, "Created", None)?),
        };

        self.post_message(&message).await?;
        tracing::info!("Posted task creation notification to Slack");
        Ok(())
    }

    pub async fn notify_task_updated(
        &self,
        task_id: TaskId,
        title: &str,
        changes: &TaskChanges,
    ) -> Result<()> {
        if !self.should_notify(&NotificationLevel::StatusChanges)
            && !self.should_notify(&NotificationLevel::All)
        {
            return Ok(());
        }

        let change_summary = self.format_changes(changes);
        let message = SlackMessage {
            channel: self.config.default_channel.clone(),
            text: format!(
                "📝 Task updated: *{}*\nChanges: {}\nTask ID: `{}`",
                title, change_summary, task_id
            ),
            thread_ts: None,
            blocks: Some(self.create_task_blocks(task_id, title, "Updated", Some(changes))?),
        };

        self.post_message(&message).await?;
        tracing::info!("Posted task update notification to Slack");
        Ok(())
    }

    pub async fn notify_task_completed(&self, task_id: TaskId, title: &str) -> Result<()> {
        if !self.should_notify(&NotificationLevel::Completions)
            && !self.should_notify(&NotificationLevel::All)
        {
            return Ok(());
        }

        let message = SlackMessage {
            channel: self.config.default_channel.clone(),
            text: format!("✅ Task completed: *{}*\nTask ID: `{}`", title, task_id),
            thread_ts: None,
            blocks: Some(self.create_task_blocks(task_id, title, "Completed", None)?),
        };

        self.post_message(&message).await?;
        tracing::info!("Posted task completion notification to Slack");
        Ok(())
    }

    pub async fn notify_task_failed(
        &self,
        task_id: TaskId,
        title: &str,
        reason: &str,
    ) -> Result<()> {
        if !self.should_notify(&NotificationLevel::BlockingEvents)
            && !self.should_notify(&NotificationLevel::All)
        {
            return Ok(());
        }

        let message = SlackMessage {
            channel: self.config.default_channel.clone(),
            text: format!(
                "❌ Task failed: *{}*\nReason: {}\nTask ID: `{}`",
                title, reason, task_id
            ),
            thread_ts: None,
            blocks: Some(self.create_failure_blocks(task_id, title, reason)?),
        };

        self.post_message(&message).await?;
        tracing::info!("Posted task failure notification to Slack");
        Ok(())
    }

    pub async fn notify_session_assignment(
        &self,
        task_id: TaskId,
        title: &str,
        session: &str,
        user: &str,
    ) -> Result<()> {
        let message = SlackMessage {
            channel: self.config.default_channel.clone(),
            text: format!(
                "👤 Task assigned: *{}*\nSession: {}\nAssigned to: <@{}>\nTask ID: `{}`",
                title, session, user, task_id
            ),
            thread_ts: None,
            blocks: Some(self.create_assignment_blocks(task_id, title, session, user)?),
        };

        self.post_message(&message).await?;
        tracing::info!("Posted session assignment notification to Slack");
        Ok(())
    }

    pub async fn notify_dependency_blocking(
        &self,
        blocked_task_id: TaskId,
        blocking_task_id: TaskId,
    ) -> Result<()> {
        if !self.should_notify(&NotificationLevel::BlockingEvents)
            && !self.should_notify(&NotificationLevel::All)
        {
            return Ok(());
        }

        let message = SlackMessage {
            channel: self.config.default_channel.clone(),
            text: format!(
                "🚧 Task blocked: `{}` is waiting on `{}`",
                blocked_task_id, blocking_task_id
            ),
            thread_ts: None,
            blocks: Some(self.create_dependency_blocks(blocked_task_id, blocking_task_id)?),
        };

        self.post_message(&message).await?;
        tracing::info!("Posted dependency blocking notification to Slack");
        Ok(())
    }

    pub async fn handle_event(&self, payload: SlackEventPayload) -> Result<String> {
        // Handle URL verification challenge
        if let Some(challenge) = payload.challenge {
            return Ok(challenge);
        }

        if let Some(event) = payload.event {
            match event.type_.as_str() {
                "message" => {
                    self.handle_message_event(event).await?;
                }
                "app_mention" => {
                    self.handle_mention_event(event).await?;
                }
                _ => {
                    tracing::debug!("Unhandled Slack event type: {}", event.type_);
                }
            }
        }

        Ok("ok".to_string())
    }

    async fn handle_message_event(&self, _event: SlackEvent) -> Result<()> {
        tracing::info!("Handling Slack message event");
        // TODO: Parse message for task commands or updates
        Ok(())
    }

    async fn handle_mention_event(&self, _event: SlackEvent) -> Result<()> {
        tracing::info!("Handling Slack mention event");
        // TODO: Respond to mentions with task status or help
        Ok(())
    }

    fn should_notify(&self, level: &NotificationLevel) -> bool {
        self.config.notification_levels.contains(level)
            || self
                .config
                .notification_levels
                .contains(&NotificationLevel::All)
    }

    fn format_changes(&self, changes: &TaskChanges) -> String {
        let mut parts = Vec::new();

        if let Some(status) = &changes.status {
            parts.push(format!("status → {}", status));
        }
        if let Some(title) = &changes.title {
            parts.push(format!("title → {}", title));
        }
        if let Some(priority) = &changes.priority {
            parts.push(format!("priority → {}", priority));
        }
        if let Some(assignee) = &changes.assignee {
            parts.push(format!("assignee → {}", assignee));
        }

        parts.join(", ")
    }

    fn create_task_blocks(
        &self,
        task_id: TaskId,
        title: &str,
        action: &str,
        changes: Option<&TaskChanges>,
    ) -> Result<serde_json::Value> {
        let mut fields = vec![
            serde_json::json!({
                "type": "mrkdwn",
                "text": format!("*Task ID:*\n`{}`", task_id)
            }),
            serde_json::json!({
                "type": "mrkdwn",
                "text": format!("*Action:*\n{}", action)
            }),
        ];

        if let Some(changes) = changes {
            let change_text = self.format_changes(changes);
            if !change_text.is_empty() {
                fields.push(serde_json::json!({
                    "type": "mrkdwn",
                    "text": format!("*Changes:*\n{}", change_text)
                }));
            }
        }

        Ok(serde_json::json!([
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": format!("*{}*", title)
                }
            },
            {
                "type": "section",
                "fields": fields
            }
        ]))
    }

    fn create_failure_blocks(
        &self,
        task_id: TaskId,
        title: &str,
        reason: &str,
    ) -> Result<serde_json::Value> {
        Ok(serde_json::json!([
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": format!("❌ *Task Failed: {}*", title)
                }
            },
            {
                "type": "section",
                "fields": [
                    {
                        "type": "mrkdwn",
                        "text": format!("*Task ID:*\n`{}`", task_id)
                    },
                    {
                        "type": "mrkdwn",
                        "text": format!("*Reason:*\n{}", reason)
                    }
                ]
            }
        ]))
    }

    fn create_assignment_blocks(
        &self,
        task_id: TaskId,
        title: &str,
        session: &str,
        user: &str,
    ) -> Result<serde_json::Value> {
        Ok(serde_json::json!([
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": format!("👤 *Task Assigned: {}*", title)
                }
            },
            {
                "type": "section",
                "fields": [
                    {
                        "type": "mrkdwn",
                        "text": format!("*Task ID:*\n`{}`", task_id)
                    },
                    {
                        "type": "mrkdwn",
                        "text": format!("*Session:*\n{}", session)
                    },
                    {
                        "type": "mrkdwn",
                        "text": format!("*Assigned to:*\n<@{}>", user)
                    }
                ]
            }
        ]))
    }

    fn create_dependency_blocks(
        &self,
        blocked_task_id: TaskId,
        blocking_task_id: TaskId,
    ) -> Result<serde_json::Value> {
        Ok(serde_json::json!([
            {
                "type": "section",
                "text": {
                    "type": "mrkdwn",
                    "text": "🚧 *Task Dependency Blocking*"
                }
            },
            {
                "type": "section",
                "fields": [
                    {
                        "type": "mrkdwn",
                        "text": format!("*Blocked Task:*\n`{}`", blocked_task_id)
                    },
                    {
                        "type": "mrkdwn",
                        "text": format!("*Waiting On:*\n`{}`", blocking_task_id)
                    }
                ]
            }
        ]))
    }

    pub fn verify_signature(&self, timestamp: &str, body: &str, signature: &str) -> bool {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;

        type HmacSha256 = Hmac<Sha256>;

        let basestring = format!("v0:{}:{}", timestamp, body);

        let mut mac = match HmacSha256::new_from_slice(self.config.signing_secret.as_bytes()) {
            Ok(mac) => mac,
            Err(_) => return false,
        };

        mac.update(basestring.as_bytes());

        let expected = format!("v0={}", hex::encode(mac.finalize().into_bytes()));

        signature == expected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_config() -> SlackConfig {
        SlackConfig {
            bot_token: "xoxb-test-token".to_string(),
            signing_secret: "test_secret".to_string(),
            default_channel: "#test-channel".to_string(),
            notification_levels: vec![NotificationLevel::All],
        }
    }

    #[test]
    fn test_slack_integration_creation() {
        let config = create_test_config();
        let event_bus = IntegrationEventBus::new();
        let integration = SlackIntegration::new(config.clone(), event_bus);

        assert_eq!(integration.config.default_channel, "#test-channel");
    }

    #[test]
    fn test_notification_level_check() {
        let config = create_test_config();
        let event_bus = IntegrationEventBus::new();
        let integration = SlackIntegration::new(config, event_bus);

        assert!(integration.should_notify(&NotificationLevel::StatusChanges));
        assert!(integration.should_notify(&NotificationLevel::Completions));
        assert!(integration.should_notify(&NotificationLevel::BlockingEvents));
    }

    #[test]
    fn test_notification_level_specific() {
        let config = SlackConfig {
            bot_token: "xoxb-test-token".to_string(),
            signing_secret: "test_secret".to_string(),
            default_channel: "#test-channel".to_string(),
            notification_levels: vec![NotificationLevel::Completions],
        };
        let event_bus = IntegrationEventBus::new();
        let integration = SlackIntegration::new(config, event_bus);

        assert!(!integration.should_notify(&NotificationLevel::StatusChanges));
        assert!(integration.should_notify(&NotificationLevel::Completions));
        assert!(!integration.should_notify(&NotificationLevel::BlockingEvents));
    }

    #[test]
    fn test_format_changes() {
        let config = create_test_config();
        let event_bus = IntegrationEventBus::new();
        let integration = SlackIntegration::new(config, event_bus);

        let changes = TaskChanges {
            status: Some("Completed".to_string()),
            title: Some("Updated Title".to_string()),
            description: None,
            priority: Some("High".to_string()),
            assignee: None,
        };

        let formatted = integration.format_changes(&changes);
        assert!(formatted.contains("status → Completed"));
        assert!(formatted.contains("title → Updated Title"));
        assert!(formatted.contains("priority → High"));
        assert!(!formatted.contains("description"));
        assert!(!formatted.contains("assignee"));
    }

    #[test]
    fn test_slack_message_serialization() {
        let message = SlackMessage {
            channel: "#test".to_string(),
            text: "Test message".to_string(),
            thread_ts: Some("1234567890.123456".to_string()),
            blocks: None,
        };

        let serialized = serde_json::to_string(&message).unwrap();
        let deserialized: SlackMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(message.channel, deserialized.channel);
        assert_eq!(message.text, deserialized.text);
        assert_eq!(message.thread_ts, deserialized.thread_ts);
    }

    #[test]
    fn test_slack_event_payload_deserialization() {
        let payload_json = r#"{
            "token": "test_token",
            "type": "event_callback",
            "event": {
                "type": "message",
                "user": "U123456",
                "text": "Hello world",
                "channel": "C123456",
                "ts": "1234567890.123456"
            }
        }"#;

        let payload: SlackEventPayload = serde_json::from_str(payload_json).unwrap();
        assert_eq!(payload.token, "test_token");
        assert_eq!(payload.type_, "event_callback");

        let event = payload.event.unwrap();
        assert_eq!(event.type_, "message");
        assert_eq!(event.user.unwrap(), "U123456");
        assert_eq!(event.text.unwrap(), "Hello world");
    }

    #[test]
    fn test_create_task_blocks() {
        let config = create_test_config();
        let event_bus = IntegrationEventBus::new();
        let integration = SlackIntegration::new(config, event_bus);

        let task_id = Uuid::new_v4();
        let blocks = integration
            .create_task_blocks(task_id, "Test Task", "Created", None)
            .unwrap();

        let blocks_array = blocks.as_array().unwrap();
        assert_eq!(blocks_array.len(), 2);

        let section = &blocks_array[0];
        assert_eq!(section["type"], "section");
        assert!(section["text"]["text"]
            .as_str()
            .unwrap()
            .contains("Test Task"));
    }

    #[test]
    fn test_url_verification_challenge() {
        let payload = SlackEventPayload {
            token: "test_token".to_string(),
            challenge: Some("challenge_response".to_string()),
            event: None,
            type_: "url_verification".to_string(),
        };

        assert_eq!(payload.challenge.unwrap(), "challenge_response");
    }
}
