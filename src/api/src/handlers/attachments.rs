use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use luce_shared::{
    AttachmentData, GitHubAttachment, GitHubPRState, LuceError, TaskAttachment, TaskId,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::services::TaskService;

#[derive(Serialize, Deserialize)]
pub struct CreateGitHubAttachmentRequest {
    pub issue_number: Option<u32>,
    pub pr_number: Option<u64>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub head_branch: Option<String>,
    pub base_branch: Option<String>,
    pub draft: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct AttachmentResponse {
    pub id: String,
    pub task_id: TaskId,
    pub attachment_type: String,
    pub title: String,
    pub url: String,
    pub identifier: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct ListAttachmentsQuery {
    pub attachment_type: Option<String>,
}

impl From<&TaskAttachment> for AttachmentResponse {
    fn from(attachment: &TaskAttachment) -> Self {
        Self {
            id: attachment.id.to_string(),
            task_id: attachment.task_id,
            attachment_type: attachment.attachment_type().to_string(),
            title: attachment.title().to_string(),
            url: attachment.url().to_string(),
            identifier: attachment.identifier(),
            created_at: attachment.created_at,
            updated_at: attachment.updated_at,
        }
    }
}

pub async fn list_attachments(
    Extension(service): Extension<Arc<TaskService>>,
    Path(task_id): Path<String>,
    Query(params): Query<ListAttachmentsQuery>,
) -> Result<Json<Vec<AttachmentResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let id = parse_task_id(&task_id)?;

    // TODO: Implement attachment storage and retrieval
    // For now, return empty list with proper structure
    let attachments: Vec<AttachmentResponse> = Vec::new();
    Ok(Json(attachments))
}

pub async fn create_github_attachment(
    Extension(service): Extension<Arc<TaskService>>,
    Path(task_id): Path<String>,
    Json(request): Json<CreateGitHubAttachmentRequest>,
) -> Result<(StatusCode, Json<AttachmentResponse>), (StatusCode, Json<serde_json::Value>)> {
    let id = parse_task_id(&task_id)?;

    // Validate request
    if request.issue_number.is_none() && request.pr_number.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Either issue_number or pr_number must be provided"
            })),
        ));
    }

    if request.issue_number.is_some() && request.pr_number.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Cannot specify both issue_number and pr_number"
            })),
        ));
    }

    // TODO: Implement GitHub integration
    // For now, create a mock attachment
    let mock_attachment = if let Some(pr_number) = request.pr_number {
        let github_data = GitHubAttachment {
            repository: "example/repo".to_string(),
            pr_number,
            title: request
                .title
                .unwrap_or_else(|| format!("PR #{}", pr_number)),
            state: GitHubPRState::Open,
            author: "user".to_string(),
            base_branch: request.base_branch.unwrap_or_else(|| "main".to_string()),
            head_branch: request.head_branch.unwrap_or_else(|| "feature".to_string()),
            url: format!("https://github.com/example/repo/pull/{}", pr_number),
        };
        TaskAttachment::new_github(id, github_data)
    } else if let Some(issue_number) = request.issue_number {
        // For issues, we'll create a PR attachment with issue-like properties
        let github_data = GitHubAttachment {
            repository: "example/repo".to_string(),
            pr_number: issue_number as u64,
            title: request
                .title
                .unwrap_or_else(|| format!("Issue #{}", issue_number)),
            state: GitHubPRState::Open,
            author: "user".to_string(),
            base_branch: "main".to_string(),
            head_branch: "issue".to_string(),
            url: format!("https://github.com/example/repo/issues/{}", issue_number),
        };
        TaskAttachment::new_github(id, github_data)
    } else {
        unreachable!()
    };

    let response = AttachmentResponse::from(&mock_attachment);

    println!(
        "Created GitHub attachment: {} for task {}",
        response.identifier, id
    );

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn get_attachment(
    Extension(service): Extension<Arc<TaskService>>,
    Path((task_id, attachment_id)): Path<(String, String)>,
) -> Result<Json<AttachmentResponse>, (StatusCode, Json<serde_json::Value>)> {
    let task_uuid = parse_task_id(&task_id)?;
    let attachment_uuid = parse_attachment_id(&attachment_id)?;

    // TODO: Implement attachment retrieval
    Err((
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "Attachment not found"
        })),
    ))
}

pub async fn delete_attachment(
    Extension(service): Extension<Arc<TaskService>>,
    Path((task_id, attachment_id)): Path<(String, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let task_uuid = parse_task_id(&task_id)?;
    let attachment_uuid = parse_attachment_id(&attachment_id)?;

    // TODO: Implement attachment deletion
    println!("Deleted attachment {} from task {}", attachment_id, task_id);

    Ok(StatusCode::NO_CONTENT)
}

pub async fn sync_github_attachments(
    Extension(service): Extension<Arc<TaskService>>,
    Path(task_id): Path<String>,
) -> Result<Json<Vec<AttachmentResponse>>, (StatusCode, Json<serde_json::Value>)> {
    let id = parse_task_id(&task_id)?;

    // TODO: Implement GitHub synchronization
    println!("Syncing GitHub attachments for task {}", id);

    let attachments: Vec<AttachmentResponse> = Vec::new();
    Ok(Json(attachments))
}

fn parse_task_id(task_id: &str) -> Result<TaskId, (StatusCode, Json<serde_json::Value>)> {
    Uuid::parse_str(task_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid task ID format"
            })),
        )
    })
}

fn parse_attachment_id(attachment_id: &str) -> Result<Uuid, (StatusCode, Json<serde_json::Value>)> {
    Uuid::parse_str(attachment_id).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid attachment ID format"
            })),
        )
    })
}

pub fn attachment_routes() -> Router {
    Router::new()
        .route("/tasks/:task_id/attachments", get(list_attachments))
        .route(
            "/tasks/:task_id/attachments/github",
            post(create_github_attachment),
        )
        .route(
            "/tasks/:task_id/attachments/:attachment_id",
            get(get_attachment),
        )
        .route(
            "/tasks/:task_id/attachments/:attachment_id",
            delete(delete_attachment),
        )
        .route(
            "/tasks/:task_id/attachments/sync/github",
            post(sync_github_attachments),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_create_github_pr_attachment_request() {
        let request = CreateGitHubAttachmentRequest {
            issue_number: None,
            pr_number: Some(123),
            title: Some("Test PR".to_string()),
            body: Some("Test description".to_string()),
            head_branch: Some("feature/test".to_string()),
            base_branch: Some("main".to_string()),
            draft: Some(false),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CreateGitHubAttachmentRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.pr_number, deserialized.pr_number);
        assert_eq!(request.title, deserialized.title);
        assert_eq!(request.body, deserialized.body);
    }

    #[tokio::test]
    async fn test_create_github_issue_attachment_request() {
        let request = CreateGitHubAttachmentRequest {
            issue_number: Some(456),
            pr_number: None,
            title: Some("Test Issue".to_string()),
            body: Some("Test issue description".to_string()),
            head_branch: None,
            base_branch: None,
            draft: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CreateGitHubAttachmentRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.issue_number, deserialized.issue_number);
        assert_eq!(request.title, deserialized.title);
        assert_eq!(request.body, deserialized.body);
    }

    #[test]
    fn test_attachment_response_from_task_attachment() {
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
        let response = AttachmentResponse::from(&attachment);

        assert_eq!(response.task_id, task_id);
        assert_eq!(response.attachment_type, "github");
        assert_eq!(response.title, "Test PR");
        assert_eq!(response.url, "https://github.com/owner/repo/pull/42");
        assert_eq!(response.identifier, "owner/repo#42");
    }

    #[test]
    fn test_list_attachments_query_deserialization() {
        let json = json!({
            "attachment_type": "github"
        });

        let query: ListAttachmentsQuery = serde_json::from_value(json).unwrap();
        assert_eq!(query.attachment_type, Some("github".to_string()));
    }

    #[test]
    fn test_parse_task_id_valid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let result = parse_task_id(uuid_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_task_id_invalid() {
        let invalid_str = "not-a-uuid";
        let result = parse_task_id(invalid_str);
        assert!(result.is_err());

        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_parse_attachment_id_valid() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440001";
        let result = parse_attachment_id(uuid_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_attachment_id_invalid() {
        let invalid_str = "not-a-uuid";
        let result = parse_attachment_id(invalid_str);
        assert!(result.is_err());

        let (status, _) = result.unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}
