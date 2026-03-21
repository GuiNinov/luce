use crate::{IntegrationEventBus, TaskChanges};
use anyhow::Result;
use luce_shared::{SlackAttachment, TaskAttachment, TaskId};
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

    // Thread Attachment Methods

    /// Get conversation info for a channel
    pub async fn get_conversation_info(&self, channel_id: &str) -> Result<serde_json::Value> {
        let url = format!(
            "https://slack.com/api/conversations.info?channel={}",
            channel_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.bot_token))
            .send()
            .await?;

        let slack_response: serde_json::Value = response.json().await?;

        if !slack_response["ok"].as_bool().unwrap_or(false) {
            return Err(anyhow::anyhow!(
                "Slack API error: {:?}",
                slack_response["error"]
            ));
        }

        Ok(slack_response)
    }

    /// Get conversation history to find thread information
    pub async fn get_conversation_history(
        &self,
        channel_id: &str,
        ts: &str,
    ) -> Result<serde_json::Value> {
        let url = format!(
            "https://slack.com/api/conversations.history?channel={}&latest={}&limit=1&inclusive=true",
            channel_id, ts
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.bot_token))
            .send()
            .await?;

        let slack_response: serde_json::Value = response.json().await?;

        if !slack_response["ok"].as_bool().unwrap_or(false) {
            return Err(anyhow::anyhow!(
                "Slack API error: {:?}",
                slack_response["error"]
            ));
        }

        Ok(slack_response)
    }

    /// Get replies for a thread
    pub async fn get_thread_replies(
        &self,
        channel_id: &str,
        thread_ts: &str,
    ) -> Result<serde_json::Value> {
        let url = format!(
            "https://slack.com/api/conversations.replies?channel={}&ts={}",
            channel_id, thread_ts
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.bot_token))
            .send()
            .await?;

        let slack_response: serde_json::Value = response.json().await?;

        if !slack_response["ok"].as_bool().unwrap_or(false) {
            return Err(anyhow::anyhow!(
                "Slack API error: {:?}",
                slack_response["error"]
            ));
        }

        Ok(slack_response)
    }

    /// Create a task attachment from Slack thread information
    pub async fn create_thread_attachment(
        &self,
        task_id: TaskId,
        channel_id: &str,
        thread_ts: &str,
    ) -> Result<TaskAttachment> {
        // Get channel info
        let channel_info = self.get_conversation_info(channel_id).await?;
        let channel_name = channel_info["channel"]["name"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        // Get the original message
        let history = self.get_conversation_history(channel_id, thread_ts).await?;
        let empty_vec = vec![];
        let messages = history["messages"].as_array().unwrap_or(&empty_vec);

        if messages.is_empty() {
            return Err(anyhow::anyhow!("Thread message not found"));
        }

        let original_message = &messages[0];
        let message_text = original_message["text"].as_str().unwrap_or("").to_string();
        let author_user_id = original_message["user"].as_str().unwrap_or("unknown");

        // Get thread replies to count them
        let replies = self.get_thread_replies(channel_id, thread_ts).await?;
        let reply_count = replies["messages"]
            .as_array()
            .map(|msgs| msgs.len().saturating_sub(1)) // Subtract original message
            .unwrap_or(0) as u32;

        // Create thread URL
        let thread_url = format!(
            "https://workspace.slack.com/archives/{}/p{}",
            channel_id,
            thread_ts.replace(".", "")
        );

        // Truncate message for preview (max 100 chars)
        let message_preview = if message_text.len() > 100 {
            format!("{}...", &message_text[..100])
        } else {
            message_text
        };

        let slack_attachment = SlackAttachment {
            channel_id: channel_id.to_string(),
            channel_name,
            thread_ts: thread_ts.to_string(),
            message_preview,
            author: author_user_id.to_string(),
            reply_count,
            url: thread_url,
        };

        Ok(TaskAttachment::new_slack(task_id, slack_attachment))
    }

    /// Attach a Slack thread to a task
    pub async fn attach_thread(
        &self,
        task_id: TaskId,
        channel_id: &str,
        thread_ts: &str,
    ) -> Result<TaskAttachment> {
        self.create_thread_attachment(task_id, channel_id, thread_ts)
            .await
    }

    /// Start a new thread for a task and attach it
    pub async fn create_and_attach_thread(
        &self,
        task_id: TaskId,
        channel_id: &str,
        message_text: &str,
    ) -> Result<TaskAttachment> {
        let message = SlackMessage {
            channel: channel_id.to_string(),
            text: message_text.to_string(),
            thread_ts: None,
            blocks: None,
        };

        // Post the message to start a thread
        let response = self.post_message(&message).await?;

        let thread_ts = response["ts"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Failed to get message timestamp"))?;

        // Create attachment for the new thread
        self.create_thread_attachment(task_id, channel_id, thread_ts)
            .await
    }

    /// Update an attachment with the latest thread data
    pub async fn refresh_thread_attachment(&self, attachment: &mut TaskAttachment) -> Result<()> {
        match &attachment.data {
            luce_shared::AttachmentData::Slack(slack_data) => {
                let updated_attachment = self
                    .create_thread_attachment(
                        attachment.task_id,
                        &slack_data.channel_id,
                        &slack_data.thread_ts,
                    )
                    .await?;

                if let luce_shared::AttachmentData::Slack(updated_slack_data) =
                    updated_attachment.data
                {
                    attachment.data = luce_shared::AttachmentData::Slack(updated_slack_data);
                    attachment.touch();
                }
            }
            _ => return Err(anyhow::anyhow!("Attachment is not a Slack attachment")),
        }
        Ok(())
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

    #[test]
    fn test_create_thread_attachment_mock() {
        let event_bus = IntegrationEventBus::new();
        let config = create_test_config();
        let _integration = SlackIntegration::new(config, event_bus);
        let task_id = TaskId::new_v4();

        // This test verifies the attachment structure without making real API calls
        let slack_attachment = SlackAttachment {
            channel_id: "C1234567890".to_string(),
            channel_name: "engineering".to_string(),
            thread_ts: "1234567890.123456".to_string(),
            message_preview: "Discussion about the new feature implementation".to_string(),
            author: "U0987654321".to_string(),
            reply_count: 5,
            url: "https://workspace.slack.com/archives/C1234567890/p1234567890123456".to_string(),
        };

        let attachment = TaskAttachment::new_slack(task_id, slack_attachment.clone());

        assert_eq!(attachment.task_id, task_id);
        assert_eq!(
            attachment.title(),
            "Discussion about the new feature implementation"
        );
        assert_eq!(
            attachment.url(),
            "https://workspace.slack.com/archives/C1234567890/p1234567890123456"
        );
        assert_eq!(attachment.identifier(), "#engineering");

        match &attachment.data {
            luce_shared::AttachmentData::Slack(slack_data) => {
                assert_eq!(slack_data.channel_id, "C1234567890");
                assert_eq!(slack_data.channel_name, "engineering");
                assert_eq!(slack_data.thread_ts, "1234567890.123456");
                assert_eq!(
                    slack_data.message_preview,
                    "Discussion about the new feature implementation"
                );
                assert_eq!(slack_data.author, "U0987654321");
                assert_eq!(slack_data.reply_count, 5);
            }
            _ => panic!("Expected Slack attachment"),
        }
    }

    #[test]
    fn test_slack_message_truncation() {
        let long_message = "This is a very long message that should be truncated because it exceeds the 100 character limit for message previews in Slack attachments and we want to test this behavior properly.";

        let truncated = if long_message.len() > 100 {
            format!("{}...", &long_message[..100])
        } else {
            long_message.to_string()
        };

        assert!(truncated.len() <= 103); // 100 chars + "..."
        assert!(truncated.ends_with("..."));
        assert_eq!(&truncated[..100], &long_message[..100]);
    }

    #[test]
    fn test_slack_thread_url_generation() {
        let channel_id = "C1234567890";
        let thread_ts = "1234567890.123456";

        let expected_url = format!(
            "https://workspace.slack.com/archives/{}/p{}",
            channel_id,
            thread_ts.replace(".", "")
        );

        assert_eq!(
            expected_url,
            "https://workspace.slack.com/archives/C1234567890/p1234567890123456"
        );
    }
}
