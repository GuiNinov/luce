use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use luce_shared::{LuceError, Task, TaskId, TaskStatus};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::services::TaskService;

#[derive(Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub dependencies: Vec<TaskId>,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskResponse {
    pub id: TaskId,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub dependencies: Vec<TaskId>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct GraphResponse {
    pub tasks: Vec<TaskResponse>,
    pub total_count: usize,
}

#[derive(Deserialize)]
pub struct ListTasksQuery {
    pub status: Option<TaskStatus>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl From<&Task> for TaskResponse {
    fn from(task: &Task) -> Self {
        Self {
            id: task.id,
            title: task.title.clone(),
            description: task.description.clone(),
            status: task.status,
            dependencies: vec![], // TODO: Add dependency support
            created_at: task.created_at,
            updated_at: task.updated_at,
        }
    }
}

pub async fn create_task(
    Extension(service): Extension<Arc<TaskService>>,
    Json(request): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<TaskResponse>), (StatusCode, Json<serde_json::Value>)> {
    match service
        .create_task(request.title, request.description, request.dependencies)
        .await
    {
        Ok(task) => {
            let response = TaskResponse::from(&task);
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(e) => Err(handle_error(e)),
    }
}

pub async fn get_task(
    Extension(service): Extension<Arc<TaskService>>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, (StatusCode, Json<serde_json::Value>)> {
    let id = Uuid::parse_str(&task_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid UUID format"
            })),
        )
    })?;

    match service.get_task(id).await {
        Ok(task) => {
            let response = TaskResponse::from(&task);
            Ok(Json(response))
        }
        Err(e) => Err(handle_error(e)),
    }
}

pub async fn update_task(
    Extension(service): Extension<Arc<TaskService>>,
    Path(task_id): Path<String>,
    Json(request): Json<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, (StatusCode, Json<serde_json::Value>)> {
    let id = Uuid::parse_str(&task_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid UUID format"
            })),
        )
    })?;

    match service
        .update_task(id, request.title, request.description, request.status)
        .await
    {
        Ok(task) => {
            let response = TaskResponse::from(&task);
            Ok(Json(response))
        }
        Err(e) => Err(handle_error(e)),
    }
}

pub async fn delete_task(
    Extension(service): Extension<Arc<TaskService>>,
    Path(task_id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let id = Uuid::parse_str(&task_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid UUID format"
            })),
        )
    })?;

    match service.delete_task(id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(handle_error(e)),
    }
}

pub async fn list_tasks(
    Extension(service): Extension<Arc<TaskService>>,
    Query(params): Query<ListTasksQuery>,
) -> Result<Json<GraphResponse>, (StatusCode, Json<serde_json::Value>)> {
    match service
        .list_tasks(params.status, params.limit, params.offset)
        .await
    {
        Ok((tasks, total_count)) => {
            let task_responses: Vec<TaskResponse> = tasks.iter().map(TaskResponse::from).collect();
            let response = GraphResponse {
                tasks: task_responses,
                total_count,
            };
            Ok(Json(response))
        }
        Err(e) => Err(handle_error(e)),
    }
}

pub async fn get_ready_tasks(
    Extension(service): Extension<Arc<TaskService>>,
) -> Result<Json<Vec<TaskResponse>>, (StatusCode, Json<serde_json::Value>)> {
    match service.get_ready_tasks().await {
        Ok(tasks) => {
            let task_responses: Vec<TaskResponse> = tasks.iter().map(TaskResponse::from).collect();
            Ok(Json(task_responses))
        }
        Err(e) => Err(handle_error(e)),
    }
}

pub async fn mark_task_completed(
    Extension(service): Extension<Arc<TaskService>>,
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, (StatusCode, Json<serde_json::Value>)> {
    let id = Uuid::parse_str(&task_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid UUID format"
            })),
        )
    })?;

    match service.mark_task_completed(id).await {
        Ok(task) => {
            let response = TaskResponse::from(&task);
            Ok(Json(response))
        }
        Err(e) => Err(handle_error(e)),
    }
}

pub async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    })))
}

fn handle_error(error: LuceError) -> (StatusCode, Json<serde_json::Value>) {
    let (status, message) = match error {
        LuceError::TaskNotFound { id: _ } => (StatusCode::NOT_FOUND, "Task not found"),
        LuceError::CircularDependency => (StatusCode::BAD_REQUEST, "Circular dependency detected"),
        LuceError::InvalidStateTransition { from: _, to: _ } => {
            (StatusCode::BAD_REQUEST, "Invalid state transition")
        }
        LuceError::DependencyError { message: _ } => (StatusCode::BAD_REQUEST, "Dependency error"),
        LuceError::SerializationError(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Serialization error")
        }
        LuceError::IoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO error"),
        LuceError::InvalidTaskId(_) => (StatusCode::BAD_REQUEST, "Invalid task ID"),
        LuceError::DatabaseError { message: _ } => {
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
        }
    };

    (
        status,
        Json(serde_json::json!({
            "error": message,
            "details": error.to_string()
        })),
    )
}

pub fn task_routes() -> Router {
    Router::new()
        .route("/tasks", post(create_task))
        .route("/tasks", get(list_tasks))
        .route("/tasks/ready", get(get_ready_tasks))
        .route("/tasks/:id", get(get_task))
        .route("/tasks/:id", put(update_task))
        .route("/tasks/:id", delete(delete_task))
        .route("/tasks/:id/complete", post(mark_task_completed))
        .route("/health", get(health_check))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_create_task_request_serialization() {
        let request = CreateTaskRequest {
            title: "Test Task".to_string(),
            description: Some("Test description".to_string()),
            dependencies: vec![],
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CreateTaskRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.title, deserialized.title);
        assert_eq!(request.description, deserialized.description);
        assert_eq!(request.dependencies, deserialized.dependencies);
    }

    #[tokio::test]
    async fn test_update_task_request_serialization() {
        let request = UpdateTaskRequest {
            title: Some("Updated Title".to_string()),
            description: None,
            status: Some(TaskStatus::InProgress),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: UpdateTaskRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.title, deserialized.title);
        assert_eq!(request.description, deserialized.description);
        assert_eq!(request.status, deserialized.status);
    }

    #[tokio::test]
    async fn test_task_response_from_task() {
        let task = Task::new("Test Task".to_string()).with_description("Description".to_string());
        let response = TaskResponse::from(&task);

        assert_eq!(response.id, task.id);
        assert_eq!(response.title, task.title);
        assert_eq!(response.description, task.description);
        assert_eq!(response.status, task.status);
        assert_eq!(response.created_at, task.created_at);
        assert_eq!(response.updated_at, task.updated_at);
    }

    #[tokio::test]
    async fn test_graph_response_serialization() {
        let task = Task::new("Test".to_string());
        let response = GraphResponse {
            tasks: vec![TaskResponse::from(&task)],
            total_count: 1,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: GraphResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.total_count, deserialized.total_count);
        assert_eq!(response.tasks.len(), deserialized.tasks.len());
    }

    #[tokio::test]
    async fn test_list_tasks_query_deserialization() {
        let json = json!({
            "status": "Pending",
            "limit": 10,
            "offset": 20
        });

        let query: ListTasksQuery = serde_json::from_value(json).unwrap();

        assert_eq!(query.status, Some(TaskStatus::Pending));
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(20));
    }

    #[test]
    fn test_health_check_returns_ok() {
        tokio_test::block_on(async {
            let result = health_check().await;
            assert!(result.is_ok());

            let Json(response) = result.unwrap();
            assert_eq!(response["status"], "healthy");
            assert!(response.get("timestamp").is_some());
        });
    }

    #[test]
    fn test_create_task_with_dependencies() {
        tokio_test::block_on(async {
            let task_id = Uuid::new_v4();
            let request = CreateTaskRequest {
                title: "Dependent Task".to_string(),
                description: Some("A task with dependencies".to_string()),
                dependencies: vec![task_id],
            };

            let result = create_task(Json(request)).await;
            assert!(result.is_ok());

            let (status, Json(response)) = result.unwrap();
            assert_eq!(status, StatusCode::CREATED);
            assert_eq!(response.title, "Dependent Task");
        });
    }

    #[test]
    fn test_get_task_with_valid_uuid() {
        tokio_test::block_on(async {
            let task_id = "550e8400-e29b-41d4-a716-446655440000";
            let result = get_task(Path(task_id.to_string())).await;
            assert!(result.is_ok());
        });
    }

    #[test]
    fn test_get_task_with_invalid_uuid() {
        tokio_test::block_on(async {
            let invalid_id = "invalid-uuid";
            let result = get_task(Path(invalid_id.to_string())).await;
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), StatusCode::BAD_REQUEST);
        });
    }

    #[test]
    fn test_delete_task_returns_no_content() {
        tokio_test::block_on(async {
            let task_id = "550e8400-e29b-41d4-a716-446655440000";
            let result = delete_task(Path(task_id.to_string())).await;
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), StatusCode::NO_CONTENT);
        });
    }

    #[test]
    fn test_list_tasks_returns_empty_response() {
        tokio_test::block_on(async {
            let query = ListTasksQuery {
                status: None,
                limit: None,
                offset: None,
            };

            let result = list_tasks(Query(query)).await;
            assert!(result.is_ok());

            let Json(response) = result.unwrap();
            assert_eq!(response.total_count, 0);
            assert!(response.tasks.is_empty());
        });
    }
}
