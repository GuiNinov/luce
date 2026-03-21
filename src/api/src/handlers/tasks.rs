use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use luce_shared::{Task, TaskId, TaskStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
            status: task.status.clone(),
            dependencies: task.dependencies.iter().cloned().collect(),
            created_at: task.created_at,
            updated_at: task.updated_at,
        }
    }
}

pub async fn create_task(
    Json(request): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<TaskResponse>), StatusCode> {
    let mut task = Task::new(request.title);

    if let Some(description) = request.description {
        task = task.with_description(description);
    }

    for dep_id in request.dependencies {
        task.dependencies.insert(dep_id);
    }

    let response = TaskResponse::from(&task);
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_task(Path(task_id): Path<String>) -> Result<Json<TaskResponse>, StatusCode> {
    let _id = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let task = Task::new("Placeholder Task".to_string());
    let response = TaskResponse::from(&task);
    Ok(Json(response))
}

pub async fn update_task(
    Path(task_id): Path<String>,
    Json(_request): Json<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let _id = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let task = Task::new("Updated Placeholder Task".to_string());
    let response = TaskResponse::from(&task);
    Ok(Json(response))
}

pub async fn delete_task(Path(task_id): Path<String>) -> Result<StatusCode, StatusCode> {
    let _id = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_tasks(
    Query(_params): Query<ListTasksQuery>,
) -> Result<Json<GraphResponse>, StatusCode> {
    let tasks = vec![];
    let response = GraphResponse {
        tasks,
        total_count: 0,
    };

    Ok(Json(response))
}

pub async fn get_ready_tasks() -> Result<Json<Vec<TaskResponse>>, StatusCode> {
    let ready_tasks = vec![];
    Ok(Json(ready_tasks))
}

pub async fn mark_task_completed(
    Path(task_id): Path<String>,
) -> Result<Json<TaskResponse>, StatusCode> {
    let _id = Uuid::parse_str(&task_id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let task = Task::new("Completed Placeholder Task".to_string());
    let response = TaskResponse::from(&task);
    Ok(Json(response))
}

pub async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    })))
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
