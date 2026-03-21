use crate::{IntegrationEventBus, TaskChanges};
use anyhow::Result;
use luce_shared::TaskId;
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
}
