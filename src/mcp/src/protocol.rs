use luce_shared::{Task, TaskGraph, TaskId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum McpRequest {
    #[serde(rename = "tasks/list")]
    ListTasks,

    #[serde(rename = "tasks/create")]
    CreateTask { params: CreateTaskParams },

    #[serde(rename = "tasks/get")]
    GetTask { params: GetTaskParams },

    #[serde(rename = "tasks/update")]
    UpdateTask { params: UpdateTaskParams },

    #[serde(rename = "tasks/delete")]
    DeleteTask { params: DeleteTaskParams },

    #[serde(rename = "graph/get")]
    GetGraph,

    #[serde(rename = "graph/ready")]
    GetReadyTasks,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskParams {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<luce_shared::TaskPriority>,
    pub dependencies: Option<Vec<TaskId>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTaskParams {
    pub id: TaskId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTaskParams {
    pub id: TaskId,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<luce_shared::TaskStatus>,
    pub priority: Option<luce_shared::TaskPriority>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteTaskParams {
    pub id: TaskId,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpResponse {
    Success(Box<SuccessResponse>),
    Error(ErrorResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub result: ResponseResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: McpError,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseResult {
    Task(Box<Task>),
    Tasks(Vec<Task>),
    Graph(TaskGraph),
    Empty,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl McpError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(code: i32, message: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new(-32602, message)
    }

    pub fn method_not_found() -> Self {
        Self::new(-32601, "Method not found")
    }

    pub fn parse_error() -> Self {
        Self::new(-32700, "Parse error")
    }

    pub fn internal_error() -> Self {
        Self::new(-32603, "Internal error")
    }

    pub fn task_not_found(id: TaskId) -> Self {
        Self::new(1001, format!("Task not found: {}", id))
    }

    pub fn dependency_cycle() -> Self {
        Self::new(1002, "Dependency cycle detected")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luce_shared::{TaskPriority, TaskStatus};
    use uuid::Uuid;

    #[test]
    fn test_mcp_error_creation() {
        let error = McpError::new(123, "Test error");
        assert_eq!(error.code, 123);
        assert_eq!(error.message, "Test error");
        assert!(error.data.is_none());
    }

    #[test]
    fn test_mcp_error_with_data() {
        let data = serde_json::json!({"key": "value"});
        let error = McpError::with_data(456, "Error with data", data.clone());
        assert_eq!(error.code, 456);
        assert_eq!(error.message, "Error with data");
        assert_eq!(error.data, Some(data));
    }

    #[test]
    fn test_predefined_errors() {
        let invalid = McpError::invalid_params("Invalid parameters");
        assert_eq!(invalid.code, -32602);

        let not_found = McpError::method_not_found();
        assert_eq!(not_found.code, -32601);

        let parse = McpError::parse_error();
        assert_eq!(parse.code, -32700);

        let internal = McpError::internal_error();
        assert_eq!(internal.code, -32603);

        let task_id = Uuid::new_v4();
        let task_error = McpError::task_not_found(task_id);
        assert_eq!(task_error.code, 1001);

        let cycle = McpError::dependency_cycle();
        assert_eq!(cycle.code, 1002);
    }

    #[test]
    fn test_create_task_params_serialization() {
        let params = CreateTaskParams {
            title: "Test Task".to_string(),
            description: Some("Description".to_string()),
            priority: Some(TaskPriority::High),
            dependencies: Some(vec![Uuid::new_v4()]),
        };

        let serialized = serde_json::to_string(&params).unwrap();
        let deserialized: CreateTaskParams = serde_json::from_str(&serialized).unwrap();

        assert_eq!(params.title, deserialized.title);
        assert_eq!(params.description, deserialized.description);
        assert_eq!(params.priority, deserialized.priority);
        assert_eq!(params.dependencies, deserialized.dependencies);
    }

    #[test]
    fn test_update_task_params_serialization() {
        let task_id = Uuid::new_v4();
        let params = UpdateTaskParams {
            id: task_id,
            title: Some("Updated Title".to_string()),
            description: Some("Updated Description".to_string()),
            status: Some(TaskStatus::InProgress),
            priority: Some(TaskPriority::Critical),
        };

        let serialized = serde_json::to_string(&params).unwrap();
        let deserialized: UpdateTaskParams = serde_json::from_str(&serialized).unwrap();

        assert_eq!(params.id, deserialized.id);
        assert_eq!(params.title, deserialized.title);
        assert_eq!(params.description, deserialized.description);
        assert_eq!(params.status, deserialized.status);
        assert_eq!(params.priority, deserialized.priority);
    }

    #[test]
    fn test_request_serialization() {
        let create_request = McpRequest::CreateTask {
            params: CreateTaskParams {
                title: "Test".to_string(),
                description: None,
                priority: None,
                dependencies: None,
            },
        };

        let serialized = serde_json::to_string(&create_request).unwrap();
        let deserialized: McpRequest = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            McpRequest::CreateTask { params } => {
                assert_eq!(params.title, "Test");
            }
            _ => panic!("Expected CreateTask request"),
        }
    }

    #[test]
    fn test_response_serialization() {
        let task = luce_shared::Task::new("Test Task".to_string());
        let response = McpResponse::Success(Box::new(SuccessResponse {
            result: ResponseResult::Task(Box::new(task.clone())),
        }));

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: McpResponse = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Task(deserialized_task) => {
                    assert_eq!(task.title, deserialized_task.title);
                    assert_eq!(task.id, deserialized_task.id);
                }
                _ => panic!("Expected Task result"),
            },
            _ => panic!("Expected Success response"),
        }
    }

    #[test]
    fn test_error_response_serialization() {
        let error = McpError::task_not_found(Uuid::new_v4());
        let response = McpResponse::Error(ErrorResponse { error });

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: McpResponse = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            McpResponse::Error(err) => {
                assert_eq!(err.error.code, 1001);
                assert!(err.error.message.contains("Task not found"));
            }
            _ => panic!("Expected Error response"),
        }
    }
}
