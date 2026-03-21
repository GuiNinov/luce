use crate::{IntegrationEventBus, TaskChanges};
use anyhow::Result;
use luce_shared::{
    LinearAttachment, LinearIssueState as SharedLinearIssueState, TaskAttachment, TaskId,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearConfig {
    pub api_key: String,
    pub webhook_secret: String,
    pub team_id: String,
    pub default_project_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearIssue {
    pub id: String,
    pub identifier: String, // e.g., "LIN-123"
    pub title: String,
    pub description: Option<String>,
    pub priority: LinearPriority,
    pub state: LinearState,
    pub assignee: Option<LinearUser>,
    pub team: LinearTeam,
    pub project: Option<LinearProject>,
    pub cycle: Option<LinearCycle>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinearPriority {
    #[serde(rename = "0")]
    NoPriority = 0,
    #[serde(rename = "1")]
    Low = 1,
    #[serde(rename = "2")]
    Medium = 2,
    #[serde(rename = "3")]
    High = 3,
    #[serde(rename = "4")]
    Urgent = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearState {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: LinearStateType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinearStateType {
    #[serde(rename = "backlog")]
    Backlog,
    #[serde(rename = "unstarted")]
    Unstarted,
    #[serde(rename = "started")]
    Started,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "canceled")]
    Canceled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearUser {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearTeam {
    pub id: String,
    pub name: String,
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearProject {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinearCycle {
    pub id: String,
    pub name: String,
    pub number: u32,
}

#[derive(Debug, Serialize)]
struct LinearGraphQLQuery {
    query: String,
    variables: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct LinearGraphQLResponse {
    data: Option<serde_json::Value>,
    errors: Option<Vec<LinearGraphQLError>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct LinearGraphQLError {
    message: String,
    locations: Option<Vec<serde_json::Value>>,
    path: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone)]
pub struct LinearIntegration {
    config: LinearConfig,
    client: Client,
    #[allow(dead_code)]
    event_bus: IntegrationEventBus,
}

impl LinearIntegration {
    pub fn new(config: LinearConfig, event_bus: IntegrationEventBus) -> Self {
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
        description: Option<&str>,
    ) -> Result<LinearIssue> {
        let mutation = r#"
            mutation IssueCreate($input: IssueCreateInput!) {
                issueCreate(input: $input) {
                    success
                    issue {
                        id
                        identifier
                        title
                        description
                        priority
                        state {
                            id
                            name
                            type
                        }
                        assignee {
                            id
                            name
                            email
                        }
                        team {
                            id
                            name
                            key
                        }
                        url
                    }
                }
            }
        "#;

        let variables = serde_json::json!({
            "input": {
                "title": title,
                "description": description,
                "teamId": self.config.team_id,
                "projectId": self.config.default_project_id
            }
        });

        let query = LinearGraphQLQuery {
            query: mutation.to_string(),
            variables,
        };

        let response = self
            .client
            .post("https://api.linear.app/graphql")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&query)
            .send()
            .await?;

        let graphql_response: LinearGraphQLResponse = response.json().await?;

        if let Some(errors) = graphql_response.errors {
            return Err(anyhow::anyhow!("Linear GraphQL errors: {:?}", errors));
        }

        let data = graphql_response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in response"))?;

        let issue_create = data
            .get("issueCreate")
            .ok_or_else(|| anyhow::anyhow!("No issueCreate in response"))?;

        let issue_data = issue_create
            .get("issue")
            .ok_or_else(|| anyhow::anyhow!("No issue in response"))?;

        let issue: LinearIssue = serde_json::from_value(issue_data.clone())?;
        Ok(issue)
    }

    pub async fn update_issue(&self, issue_id: &str, updates: &TaskChanges) -> Result<LinearIssue> {
        let mutation = r#"
            mutation IssueUpdate($id: String!, $input: IssueUpdateInput!) {
                issueUpdate(id: $id, input: $input) {
                    success
                    issue {
                        id
                        identifier
                        title
                        description
                        priority
                        state {
                            id
                            name
                            type
                        }
                        assignee {
                            id
                            name
                            email
                        }
                        team {
                            id
                            name
                            key
                        }
                        url
                    }
                }
            }
        "#;

        let mut input = serde_json::Map::new();

        if let Some(title) = &updates.title {
            input.insert(
                "title".to_string(),
                serde_json::Value::String(title.clone()),
            );
        }

        if let Some(description) = &updates.description {
            input.insert(
                "description".to_string(),
                serde_json::Value::String(description.clone()),
            );
        }

        if let Some(priority) = &updates.priority {
            let linear_priority = match priority.as_str() {
                "Critical" => 4,
                "High" => 3,
                "Normal" => 2,
                "Low" => 1,
                _ => 0,
            };
            input.insert(
                "priority".to_string(),
                serde_json::Value::Number(linear_priority.into()),
            );
        }

        let variables = serde_json::json!({
            "id": issue_id,
            "input": input
        });

        let query = LinearGraphQLQuery {
            query: mutation.to_string(),
            variables,
        };

        let response = self
            .client
            .post("https://api.linear.app/graphql")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&query)
            .send()
            .await?;

        let graphql_response: LinearGraphQLResponse = response.json().await?;

        if let Some(errors) = graphql_response.errors {
            return Err(anyhow::anyhow!("Linear GraphQL errors: {:?}", errors));
        }

        let data = graphql_response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in response"))?;

        let issue_update = data
            .get("issueUpdate")
            .ok_or_else(|| anyhow::anyhow!("No issueUpdate in response"))?;

        let issue_data = issue_update
            .get("issue")
            .ok_or_else(|| anyhow::anyhow!("No issue in response"))?;

        let issue: LinearIssue = serde_json::from_value(issue_data.clone())?;
        Ok(issue)
    }

    pub async fn get_issue(&self, issue_id: &str) -> Result<LinearIssue> {
        let query = r#"
            query Issue($id: String!) {
                issue(id: $id) {
                    id
                    identifier
                    title
                    description
                    priority
                    state {
                        id
                        name
                        type
                    }
                    assignee {
                        id
                        name
                        email
                    }
                    team {
                        id
                        name
                        key
                    }
                    project {
                        id
                        name
                    }
                    cycle {
                        id
                        name
                        number
                    }
                    url
                }
            }
        "#;

        let variables = serde_json::json!({
            "id": issue_id
        });

        let query_obj = LinearGraphQLQuery {
            query: query.to_string(),
            variables,
        };

        let response = self
            .client
            .post("https://api.linear.app/graphql")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&query_obj)
            .send()
            .await?;

        let graphql_response: LinearGraphQLResponse = response.json().await?;

        if let Some(errors) = graphql_response.errors {
            return Err(anyhow::anyhow!("Linear GraphQL errors: {:?}", errors));
        }

        let data = graphql_response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in response"))?;

        let issue_data = data
            .get("issue")
            .ok_or_else(|| anyhow::anyhow!("Issue not found"))?;

        let issue: LinearIssue = serde_json::from_value(issue_data.clone())?;
        Ok(issue)
    }

    pub async fn handle_webhook(&self, payload: serde_json::Value) -> Result<()> {
        tracing::info!("Handling Linear webhook");

        if let Some(action) = payload.get("action").and_then(|a| a.as_str()) {
            match action {
                "create" => {
                    tracing::info!("Linear issue created");
                }
                "update" => {
                    tracing::info!("Linear issue updated");
                }
                "remove" => {
                    tracing::info!("Linear issue removed");
                }
                _ => {
                    tracing::debug!("Unhandled Linear action: {}", action);
                }
            }
        }

        Ok(())
    }

    pub fn task_priority_to_linear(priority: &str) -> LinearPriority {
        match priority {
            "Critical" => LinearPriority::Urgent,
            "High" => LinearPriority::High,
            "Normal" => LinearPriority::Medium,
            "Low" => LinearPriority::Low,
            _ => LinearPriority::NoPriority,
        }
    }

    pub fn linear_priority_to_task(priority: &LinearPriority) -> &'static str {
        match priority {
            LinearPriority::Urgent => "Critical",
            LinearPriority::High => "High",
            LinearPriority::Medium => "Normal",
            LinearPriority::Low => "Low",
            LinearPriority::NoPriority => "Normal",
        }
    }

    pub fn task_status_to_linear_state_type(status: &str) -> LinearStateType {
        match status {
            "Pending" => LinearStateType::Backlog,
            "Ready" => LinearStateType::Unstarted,
            "InProgress" => LinearStateType::Started,
            "Completed" => LinearStateType::Completed,
            "Failed" | "Blocked" => LinearStateType::Canceled,
            _ => LinearStateType::Backlog,
        }
    }

    pub fn linear_state_type_to_task_status(state_type: &LinearStateType) -> &'static str {
        match state_type {
            LinearStateType::Backlog => "Pending",
            LinearStateType::Unstarted => "Ready",
            LinearStateType::Started => "InProgress",
            LinearStateType::Completed => "Completed",
            LinearStateType::Canceled => "Failed",
        }
    }

    // Issue Attachment Methods

    /// Convert Linear state type to shared LinearIssueState
    fn map_linear_state_type(state_type: &LinearStateType) -> SharedLinearIssueState {
        match state_type {
            LinearStateType::Backlog => SharedLinearIssueState::Backlog,
            LinearStateType::Unstarted => SharedLinearIssueState::Todo,
            LinearStateType::Started => SharedLinearIssueState::InProgress,
            LinearStateType::Completed => SharedLinearIssueState::Done,
            LinearStateType::Canceled => SharedLinearIssueState::Canceled,
        }
    }

    /// Convert Linear priority to string representation
    fn map_linear_priority(priority: &LinearPriority) -> String {
        match priority {
            LinearPriority::NoPriority => "None".to_string(),
            LinearPriority::Low => "Low".to_string(),
            LinearPriority::Medium => "Medium".to_string(),
            LinearPriority::High => "High".to_string(),
            LinearPriority::Urgent => "Urgent".to_string(),
        }
    }

    /// Create a task attachment from a Linear issue
    pub fn create_issue_attachment(&self, task_id: TaskId, issue: &LinearIssue) -> TaskAttachment {
        let linear_attachment = LinearAttachment {
            issue_id: issue.id.clone(),
            identifier: issue.identifier.clone(),
            title: issue.title.clone(),
            state: Self::map_linear_state_type(&issue.state.type_),
            team_id: issue.team.id.clone(),
            assignee: issue.assignee.as_ref().map(|u| u.email.clone()),
            priority: Some(Self::map_linear_priority(&issue.priority)),
            url: issue.url.clone(),
        };

        TaskAttachment::new_linear(task_id, linear_attachment)
    }

    /// Get a Linear issue by identifier (e.g., "ENG-123")
    pub async fn get_issue_by_identifier(&self, identifier: &str) -> Result<LinearIssue> {
        let query = r#"
            query IssueByIdentifier($identifier: String!) {
                issue(identifier: $identifier) {
                    id
                    identifier
                    title
                    description
                    priority
                    state {
                        id
                        name
                        type
                    }
                    assignee {
                        id
                        name
                        email
                    }
                    team {
                        id
                        name
                        key
                    }
                    url
                }
            }
        "#;

        let variables = serde_json::json!({
            "identifier": identifier
        });

        let request_body = LinearGraphQLQuery {
            query: query.to_string(),
            variables,
        };

        let response = self
            .client
            .post("https://api.linear.app/graphql")
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let graphql_response: LinearGraphQLResponse = response.json().await?;

        if let Some(errors) = graphql_response.errors {
            return Err(anyhow::anyhow!("GraphQL errors: {:?}", errors));
        }

        let data = graphql_response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data in response"))?;

        let issue_value = data
            .get("issue")
            .ok_or_else(|| anyhow::anyhow!("No issue in response data"))?;

        if issue_value.is_null() {
            return Err(anyhow::anyhow!("Issue not found: {}", identifier));
        }

        let issue: LinearIssue = serde_json::from_value(issue_value.clone())?;
        Ok(issue)
    }

    /// Attach a Linear issue to a task by identifier
    pub async fn attach_issue(
        &self,
        task_id: TaskId,
        issue_identifier: &str,
    ) -> Result<TaskAttachment> {
        let issue = self.get_issue_by_identifier(issue_identifier).await?;
        let attachment = self.create_issue_attachment(task_id, &issue);
        Ok(attachment)
    }

    /// Create a Linear issue and immediately attach it to a task
    pub async fn create_and_attach_issue(
        &self,
        task_id: TaskId,
        title: &str,
        description: Option<&str>,
    ) -> Result<TaskAttachment> {
        let issue = self.create_issue(task_id, title, description).await?;
        let attachment = self.create_issue_attachment(task_id, &issue);
        Ok(attachment)
    }

    /// Update an attachment with the latest issue data
    pub async fn refresh_issue_attachment(&self, attachment: &mut TaskAttachment) -> Result<()> {
        match &attachment.data {
            luce_shared::AttachmentData::Linear(linear_data) => {
                let issue = self
                    .get_issue_by_identifier(&linear_data.identifier)
                    .await?;

                // Create a new attachment with updated data and merge it
                let updated_attachment = self.create_issue_attachment(attachment.task_id, &issue);
                if let luce_shared::AttachmentData::Linear(updated_linear_data) =
                    updated_attachment.data
                {
                    attachment.data = luce_shared::AttachmentData::Linear(updated_linear_data);
                    attachment.touch();
                }
            }
            _ => return Err(anyhow::anyhow!("Attachment is not a Linear attachment")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> LinearConfig {
        LinearConfig {
            api_key: "test_key".to_string(),
            webhook_secret: "test_secret".to_string(),
            team_id: "test_team".to_string(),
            default_project_id: Some("test_project".to_string()),
        }
    }

    #[test]
    fn test_linear_integration_creation() {
        let config = create_test_config();
        let event_bus = IntegrationEventBus::new();
        let integration = LinearIntegration::new(config.clone(), event_bus);

        assert_eq!(integration.config.team_id, "test_team");
    }

    #[test]
    fn test_priority_conversion() {
        assert_eq!(
            LinearIntegration::linear_priority_to_task(&LinearPriority::Urgent),
            "Critical"
        );
        assert_eq!(
            LinearIntegration::linear_priority_to_task(&LinearPriority::High),
            "High"
        );
        assert_eq!(
            LinearIntegration::linear_priority_to_task(&LinearPriority::Medium),
            "Normal"
        );
        assert_eq!(
            LinearIntegration::linear_priority_to_task(&LinearPriority::Low),
            "Low"
        );
        assert_eq!(
            LinearIntegration::linear_priority_to_task(&LinearPriority::NoPriority),
            "Normal"
        );

        assert!(matches!(
            LinearIntegration::task_priority_to_linear("Critical"),
            LinearPriority::Urgent
        ));
        assert!(matches!(
            LinearIntegration::task_priority_to_linear("High"),
            LinearPriority::High
        ));
        assert!(matches!(
            LinearIntegration::task_priority_to_linear("Normal"),
            LinearPriority::Medium
        ));
        assert!(matches!(
            LinearIntegration::task_priority_to_linear("Low"),
            LinearPriority::Low
        ));
        assert!(matches!(
            LinearIntegration::task_priority_to_linear("Unknown"),
            LinearPriority::NoPriority
        ));
    }

    #[test]
    fn test_status_conversion() {
        assert_eq!(
            LinearIntegration::linear_state_type_to_task_status(&LinearStateType::Backlog),
            "Pending"
        );
        assert_eq!(
            LinearIntegration::linear_state_type_to_task_status(&LinearStateType::Unstarted),
            "Ready"
        );
        assert_eq!(
            LinearIntegration::linear_state_type_to_task_status(&LinearStateType::Started),
            "InProgress"
        );
        assert_eq!(
            LinearIntegration::linear_state_type_to_task_status(&LinearStateType::Completed),
            "Completed"
        );
        assert_eq!(
            LinearIntegration::linear_state_type_to_task_status(&LinearStateType::Canceled),
            "Failed"
        );

        assert!(matches!(
            LinearIntegration::task_status_to_linear_state_type("Pending"),
            LinearStateType::Backlog
        ));
        assert!(matches!(
            LinearIntegration::task_status_to_linear_state_type("Ready"),
            LinearStateType::Unstarted
        ));
        assert!(matches!(
            LinearIntegration::task_status_to_linear_state_type("InProgress"),
            LinearStateType::Started
        ));
        assert!(matches!(
            LinearIntegration::task_status_to_linear_state_type("Completed"),
            LinearStateType::Completed
        ));
        assert!(matches!(
            LinearIntegration::task_status_to_linear_state_type("Failed"),
            LinearStateType::Canceled
        ));
    }

    #[test]
    fn test_linear_issue_serialization() {
        let issue = LinearIssue {
            id: "issue_123".to_string(),
            identifier: "LIN-123".to_string(),
            title: "Test Issue".to_string(),
            description: Some("Test description".to_string()),
            priority: LinearPriority::High,
            state: LinearState {
                id: "state_456".to_string(),
                name: "In Progress".to_string(),
                type_: LinearStateType::Started,
            },
            assignee: Some(LinearUser {
                id: "user_789".to_string(),
                name: "Test User".to_string(),
                email: "test@example.com".to_string(),
            }),
            team: LinearTeam {
                id: "team_101".to_string(),
                name: "Test Team".to_string(),
                key: "TEST".to_string(),
            },
            project: None,
            cycle: None,
            url: "https://linear.app/test/issue/LIN-123".to_string(),
        };

        let serialized = serde_json::to_string(&issue).unwrap();
        let deserialized: LinearIssue = serde_json::from_str(&serialized).unwrap();

        assert_eq!(issue.id, deserialized.id);
        assert_eq!(issue.identifier, deserialized.identifier);
        assert_eq!(issue.title, deserialized.title);
    }

    #[test]
    fn test_graphql_query_serialization() {
        let query = LinearGraphQLQuery {
            query: "query { viewer { id } }".to_string(),
            variables: serde_json::json!({"test": "value"}),
        };

        let serialized = serde_json::to_string(&query).unwrap();
        assert!(serialized.contains("query"));
        assert!(serialized.contains("variables"));
    }

    #[test]
    fn test_task_changes_to_linear_input() {
        let changes = TaskChanges {
            status: Some("Completed".to_string()),
            title: Some("Updated Title".to_string()),
            description: Some("Updated description".to_string()),
            priority: Some("High".to_string()),
            assignee: Some("newuser".to_string()),
        };

        assert_eq!(changes.title.as_ref().unwrap(), "Updated Title");
        assert_eq!(changes.priority.as_ref().unwrap(), "High");
    }

    #[test]
    fn test_create_issue_attachment() {
        let event_bus = IntegrationEventBus::new();
        let config = create_test_config();
        let integration = LinearIntegration::new(config, event_bus);
        let task_id = TaskId::new_v4();

        let issue = LinearIssue {
            id: "issue-123".to_string(),
            identifier: "ENG-456".to_string(),
            title: "Fix critical bug".to_string(),
            description: Some("This is a critical bug that needs fixing".to_string()),
            priority: LinearPriority::High,
            state: LinearState {
                id: "state-123".to_string(),
                name: "In Progress".to_string(),
                type_: LinearStateType::Started,
            },
            assignee: Some(LinearUser {
                id: "user-123".to_string(),
                name: "Engineer".to_string(),
                email: "engineer@example.com".to_string(),
            }),
            team: LinearTeam {
                id: "team-123".to_string(),
                name: "Engineering".to_string(),
                key: "ENG".to_string(),
            },
            project: None,
            cycle: None,
            url: "https://linear.app/team/issue/ENG-456".to_string(),
        };

        let attachment = integration.create_issue_attachment(task_id, &issue);

        assert_eq!(attachment.task_id, task_id);
        assert_eq!(attachment.title(), "Fix critical bug");
        assert_eq!(attachment.url(), "https://linear.app/team/issue/ENG-456");
        assert_eq!(attachment.identifier(), "ENG-456");

        match &attachment.data {
            luce_shared::AttachmentData::Linear(linear_data) => {
                assert_eq!(linear_data.issue_id, "issue-123");
                assert_eq!(linear_data.identifier, "ENG-456");
                assert_eq!(linear_data.title, "Fix critical bug");
                assert_eq!(linear_data.state, SharedLinearIssueState::InProgress);
                assert_eq!(linear_data.team_id, "team-123");
                assert_eq!(
                    linear_data.assignee,
                    Some("engineer@example.com".to_string())
                );
                assert_eq!(linear_data.priority, Some("High".to_string()));
            }
            _ => panic!("Expected Linear attachment"),
        }
    }

    #[test]
    fn test_linear_state_mapping() {
        let test_cases = vec![
            (LinearStateType::Backlog, SharedLinearIssueState::Backlog),
            (LinearStateType::Unstarted, SharedLinearIssueState::Todo),
            (LinearStateType::Started, SharedLinearIssueState::InProgress),
            (LinearStateType::Completed, SharedLinearIssueState::Done),
            (LinearStateType::Canceled, SharedLinearIssueState::Canceled),
        ];

        for (linear_state, expected_shared_state) in test_cases {
            let mapped = LinearIntegration::map_linear_state_type(&linear_state);
            assert_eq!(mapped, expected_shared_state);
        }
    }

    #[test]
    fn test_linear_priority_mapping() {
        let test_cases = vec![
            (LinearPriority::NoPriority, "None"),
            (LinearPriority::Low, "Low"),
            (LinearPriority::Medium, "Medium"),
            (LinearPriority::High, "High"),
            (LinearPriority::Urgent, "Urgent"),
        ];

        for (linear_priority, expected_string) in test_cases {
            let mapped = LinearIntegration::map_linear_priority(&linear_priority);
            assert_eq!(mapped, expected_string);
        }
    }
}
