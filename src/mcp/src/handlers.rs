use crate::protocol::{
    AttachGitHubIssueParams, AttachGitHubPRParams, CreateGitHubIssueParams, CreateGitHubPRParams,
    CreateTaskParams, DeleteTaskParams, ErrorResponse, GetTaskParams, ListAttachmentsParams,
    McpError, McpRequest, McpResponse, RemoveAttachmentParams, ResponseResult, SuccessResponse,
    SyncGitHubParams, UpdateTaskParams,
};
use luce_core::{
    CreateTaskInput, CreateTaskUseCase, GetTaskInput, GetTaskUseCase, ListTasksInput,
    ListTasksUseCase, SqliteTaskRepository, TaskRepository, UpdateTaskStatusInput,
    UpdateTaskStatusUseCase, UseCase,
};
use luce_shared::LuceError;
use std::sync::Arc;

pub struct TaskHandler {
    task_repository: Arc<SqliteTaskRepository>,
}

impl TaskHandler {
    pub async fn new(db_path: &str) -> Result<Self, LuceError> {
        let db_url = format!("sqlite:{}", db_path);
        let task_repository = Arc::new(SqliteTaskRepository::new(&db_url).await?);

        Ok(Self { task_repository })
    }

    pub async fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request {
            McpRequest::ListTasks => self.handle_list_tasks().await,
            McpRequest::CreateTask { params } => self.handle_create_task(params).await,
            McpRequest::GetTask { params } => self.handle_get_task(params).await,
            McpRequest::UpdateTask { params } => self.handle_update_task(params).await,
            McpRequest::DeleteTask { params } => self.handle_delete_task(params).await,

            // GitHub integration handlers
            McpRequest::AttachGitHubIssue { params } => {
                self.handle_attach_github_issue(params).await
            }
            McpRequest::AttachGitHubPR { params } => self.handle_attach_github_pr(params).await,
            McpRequest::CreateGitHubIssue { params } => {
                self.handle_create_github_issue(params).await
            }
            McpRequest::CreateGitHubPR { params } => self.handle_create_github_pr(params).await,
            McpRequest::SyncGitHub { params } => self.handle_sync_github(params).await,
            McpRequest::ListAttachments { params } => self.handle_list_attachments(params).await,
            McpRequest::RemoveAttachment { params } => self.handle_remove_attachment(params).await,
        }
    }

    async fn handle_list_tasks(&self) -> McpResponse {
        let usecase = ListTasksUseCase::new(Arc::clone(&self.task_repository));
        let input = ListTasksInput::all();

        match usecase.execute(input).await {
            Ok(tasks) => McpResponse::Success(Box::new(SuccessResponse {
                result: ResponseResult::Tasks(tasks),
            })),
            Err(e) => self.handle_error(e),
        }
    }

    async fn handle_create_task(&self, params: CreateTaskParams) -> McpResponse {
        let mut input = CreateTaskInput::new(params.title);

        if let Some(desc) = params.description {
            input = input.with_description(desc);
        }

        if let Some(priority) = params.priority {
            input = input.with_priority(priority);
        }

        let usecase = CreateTaskUseCase::new(Arc::clone(&self.task_repository));

        match usecase.execute(input).await {
            Ok(task) => McpResponse::Success(Box::new(SuccessResponse {
                result: ResponseResult::Task(Box::new(task)),
            })),
            Err(e) => self.handle_error(e),
        }
    }

    async fn handle_get_task(&self, params: GetTaskParams) -> McpResponse {
        let usecase = GetTaskUseCase::new(Arc::clone(&self.task_repository));
        let input = GetTaskInput { task_id: params.id };

        match usecase.execute(input).await {
            Ok(task) => McpResponse::Success(Box::new(SuccessResponse {
                result: ResponseResult::Task(Box::new(task)),
            })),
            Err(e) => self.handle_error(e),
        }
    }

    async fn handle_update_task(&self, params: UpdateTaskParams) -> McpResponse {
        // First get the current task to update it
        let get_usecase = GetTaskUseCase::new(Arc::clone(&self.task_repository));
        let get_input = GetTaskInput { task_id: params.id };

        let mut task = match get_usecase.execute(get_input).await {
            Ok(task) => task,
            Err(e) => return self.handle_error(e),
        };

        // Update the task fields
        if let Some(title) = params.title {
            task.set_title(title);
        }

        if let Some(description) = params.description {
            task.set_description(Some(description));
        }

        if let Some(priority) = params.priority {
            task.set_priority(priority);
        }

        // Handle status updates with use case
        if let Some(status) = params.status {
            let update_status_usecase =
                UpdateTaskStatusUseCase::new(Arc::clone(&self.task_repository));
            let status_input = UpdateTaskStatusInput {
                task_id: params.id,
                new_status: status,
            };

            if let Err(e) = update_status_usecase.execute(status_input).await {
                return self.handle_error(e);
            }
            task.set_status(status);
        }

        // Save the updated task
        if let Err(e) = self.task_repository.save_task(&task).await {
            return self.handle_error(e);
        }

        McpResponse::Success(Box::new(SuccessResponse {
            result: ResponseResult::Task(Box::new(task)),
        }))
    }

    async fn handle_delete_task(&self, params: DeleteTaskParams) -> McpResponse {
        match self.task_repository.delete_task(params.id).await {
            Ok(_) => McpResponse::Success(Box::new(SuccessResponse {
                result: ResponseResult::Empty,
            })),
            Err(e) => self.handle_error(e),
        }
    }

    // GitHub integration handlers
    async fn handle_attach_github_issue(&self, params: AttachGitHubIssueParams) -> McpResponse {
        // TODO: Implement GitHub issue attachment
        // For now, return a mock response
        let result_data = serde_json::json!({
            "status": "success",
            "message": format!("GitHub issue #{} attached to task {}", params.issue_number, params.task_id),
            "attachment_type": "github_issue",
            "task_id": params.task_id,
            "issue_number": params.issue_number,
            "repository": params.repository.unwrap_or_else(|| "default".to_string())
        });

        McpResponse::Success(Box::new(SuccessResponse {
            result: ResponseResult::Value(result_data),
        }))
    }

    async fn handle_attach_github_pr(&self, params: AttachGitHubPRParams) -> McpResponse {
        // TODO: Implement GitHub PR attachment
        // For now, return a mock response
        let result_data = serde_json::json!({
            "status": "success",
            "message": format!("GitHub PR #{} attached to task {}", params.pr_number, params.task_id),
            "attachment_type": "github_pr",
            "task_id": params.task_id,
            "pr_number": params.pr_number,
            "repository": params.repository.unwrap_or_else(|| "default".to_string())
        });

        McpResponse::Success(Box::new(SuccessResponse {
            result: ResponseResult::Value(result_data),
        }))
    }

    async fn handle_create_github_issue(&self, params: CreateGitHubIssueParams) -> McpResponse {
        // TODO: Implement GitHub issue creation
        // For now, return a mock response
        let result_data = serde_json::json!({
            "status": "success",
            "message": format!("GitHub issue created for task {}", params.task_id),
            "task_id": params.task_id,
            "issue_number": 123, // Mock issue number
            "issue_url": "https://github.com/example/repo/issues/123",
            "title": params.title.unwrap_or_else(|| "Task Title".to_string()),
            "labels": params.labels.unwrap_or_default(),
            "assignees": params.assignees.unwrap_or_default()
        });

        McpResponse::Success(Box::new(SuccessResponse {
            result: ResponseResult::Value(result_data),
        }))
    }

    async fn handle_create_github_pr(&self, params: CreateGitHubPRParams) -> McpResponse {
        // TODO: Implement GitHub PR creation
        // For now, return a mock response
        let result_data = serde_json::json!({
            "status": "success",
            "message": format!("GitHub PR created for task {}", params.task_id),
            "task_id": params.task_id,
            "pr_number": 456, // Mock PR number
            "pr_url": "https://github.com/example/repo/pull/456",
            "title": params.title.unwrap_or_else(|| "Task Title".to_string()),
            "head_branch": params.head_branch,
            "base_branch": params.base_branch.unwrap_or_else(|| "main".to_string()),
            "draft": params.draft.unwrap_or(false)
        });

        McpResponse::Success(Box::new(SuccessResponse {
            result: ResponseResult::Value(result_data),
        }))
    }

    async fn handle_sync_github(&self, params: SyncGitHubParams) -> McpResponse {
        // TODO: Implement GitHub synchronization
        // For now, return a mock response
        let result_data = serde_json::json!({
            "status": "success",
            "message": "GitHub repository synchronized",
            "repository": params.repository.unwrap_or_else(|| "default".to_string()),
            "include_issues": params.include_issues.unwrap_or(true),
            "include_prs": params.include_prs.unwrap_or(true),
            "create_tasks": params.create_tasks.unwrap_or(true),
            "sync_results": {
                "issues_synced": 0,
                "prs_synced": 0,
                "tasks_created": 0,
                "tasks_updated": 0
            }
        });

        McpResponse::Success(Box::new(SuccessResponse {
            result: ResponseResult::Value(result_data),
        }))
    }

    async fn handle_list_attachments(&self, params: ListAttachmentsParams) -> McpResponse {
        // TODO: Implement attachment listing
        // For now, return a mock response
        let result_data = serde_json::json!({
            "status": "success",
            "task_id": params.task_id,
            "attachment_type_filter": params.attachment_type,
            "attachments": [] // Empty for now
        });

        McpResponse::Success(Box::new(SuccessResponse {
            result: ResponseResult::Value(result_data),
        }))
    }

    async fn handle_remove_attachment(&self, params: RemoveAttachmentParams) -> McpResponse {
        // TODO: Implement attachment removal
        // For now, return a mock response
        let result_data = serde_json::json!({
            "status": "success",
            "message": format!("Attachment {} removed from task {}", params.attachment_id, params.task_id),
            "task_id": params.task_id,
            "attachment_id": params.attachment_id
        });

        McpResponse::Success(Box::new(SuccessResponse {
            result: ResponseResult::Value(result_data),
        }))
    }

    fn handle_error(&self, error: LuceError) -> McpResponse {
        let mcp_error = match error {
            LuceError::TaskNotFound { id } => McpError::task_not_found(id),
            LuceError::CircularDependency => McpError::dependency_cycle(),
            LuceError::InvalidStateTransition { from: _, to: _ } => {
                McpError::invalid_params("Invalid state transition".to_string())
            }
            LuceError::DependencyError { message } => McpError::invalid_params(message),
            LuceError::SerializationError(_) => McpError::internal_error(),
            LuceError::IoError(_) => McpError::internal_error(),
            LuceError::InvalidTaskId(_) => {
                McpError::invalid_params("Invalid task ID format".to_string())
            }
            LuceError::DatabaseError { message } => {
                McpError::internal_error()
            }
        };

        McpResponse::Error(ErrorResponse { error: mcp_error })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luce_shared::{TaskPriority, TaskStatus};
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    async fn create_test_handler() -> TaskHandler {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();
        TaskHandler::new(db_path).await.unwrap()
    }

    #[tokio::test]
    async fn test_handler_creation() {
        let _handler = create_test_handler().await;
        // Just verify it can be created without panicking
    }

    #[tokio::test]
    async fn test_list_tasks_empty() {
        let handler = create_test_handler().await;
        let response = handler.handle_list_tasks().await;

        match response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Tasks(tasks) => assert!(tasks.is_empty()),
                _ => panic!("Expected Tasks response"),
            },
            _ => panic!("Expected success response"),
        }
    }

    #[tokio::test]
    async fn test_create_task_basic() {
        let handler = create_test_handler().await;
        let params = CreateTaskParams {
            title: "Test task".to_string(),
            description: None,
            priority: None,
            dependencies: None,
        };

        let response = handler.handle_create_task(params).await;

        match response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Task(task) => {
                    assert_eq!(task.title, "Test task");
                    assert_eq!(task.priority, TaskPriority::Normal);
                    assert!(task.description.is_none());
                }
                _ => panic!("Expected Task response"),
            },
            _ => panic!("Expected success response"),
        }
    }

    #[tokio::test]
    async fn test_create_task_with_options() {
        let handler = create_test_handler().await;
        let params = CreateTaskParams {
            title: "Test task".to_string(),
            description: Some("A test description".to_string()),
            priority: Some(TaskPriority::High),
            dependencies: None,
        };

        let response = handler.handle_create_task(params).await;

        match response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Task(task) => {
                    assert_eq!(task.title, "Test task");
                    assert_eq!(task.description, Some("A test description".to_string()));
                    assert_eq!(task.priority, TaskPriority::High);
                }
                _ => panic!("Expected Task response"),
            },
            _ => panic!("Expected success response"),
        }
    }

    #[tokio::test]
    async fn test_get_task_exists() {
        let handler = create_test_handler().await;

        // First create a task
        let create_params = CreateTaskParams {
            title: "Test task".to_string(),
            description: None,
            priority: None,
            dependencies: None,
        };

        let create_response = handler.handle_create_task(create_params).await;
        let task_id = match create_response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Task(task) => task.id,
                _ => panic!("Expected Task response"),
            },
            _ => panic!("Expected success response"),
        };

        // Now get the task
        let get_params = GetTaskParams { id: task_id };
        let response = handler.handle_get_task(get_params).await;

        match response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Task(task) => {
                    assert_eq!(task.id, task_id);
                    assert_eq!(task.title, "Test task");
                }
                _ => panic!("Expected Task response"),
            },
            _ => panic!("Expected success response"),
        }
    }

    #[tokio::test]
    async fn test_get_task_not_found() {
        let handler = create_test_handler().await;
        let nonexistent_id = Uuid::new_v4();
        let params = GetTaskParams { id: nonexistent_id };

        let response = handler.handle_get_task(params).await;

        match response {
            McpResponse::Error(err) => {
                assert_eq!(err.error.code, 1001);
                assert!(err.error.message.contains("Task not found"));
            }
            _ => panic!("Expected error response"),
        }
    }

    #[tokio::test]
    async fn test_update_task() {
        let handler = create_test_handler().await;

        // Create a task first
        let create_params = CreateTaskParams {
            title: "Original title".to_string(),
            description: None,
            priority: None,
            dependencies: None,
        };

        let task_id = match handler.handle_create_task(create_params).await {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Task(task) => task.id,
                _ => panic!("Expected Task response"),
            },
            _ => panic!("Expected success response"),
        };

        // Update the task
        let update_params = UpdateTaskParams {
            id: task_id,
            title: Some("Updated title".to_string()),
            description: Some("New description".to_string()),
            status: Some(TaskStatus::InProgress),
            priority: Some(TaskPriority::Critical),
        };

        let response = handler.handle_update_task(update_params).await;

        match response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Task(task) => {
                    assert_eq!(task.title, "Updated title");
                    assert_eq!(task.description, Some("New description".to_string()));
                    assert_eq!(task.status, TaskStatus::InProgress);
                    assert_eq!(task.priority, TaskPriority::Critical);
                }
                _ => panic!("Expected Task response"),
            },
            _ => panic!("Expected success response"),
        }
    }

    #[tokio::test]
    async fn test_update_task_not_found() {
        let handler = create_test_handler().await;
        let nonexistent_id = Uuid::new_v4();

        let params = UpdateTaskParams {
            id: nonexistent_id,
            title: Some("Updated title".to_string()),
            description: None,
            status: None,
            priority: None,
        };

        let response = handler.handle_update_task(params).await;

        match response {
            McpResponse::Error(err) => {
                assert_eq!(err.error.code, 1001);
            }
            _ => panic!("Expected error response"),
        }
    }

    #[tokio::test]
    async fn test_delete_task() {
        let handler = create_test_handler().await;

        // Create a task first
        let create_params = CreateTaskParams {
            title: "To be deleted".to_string(),
            description: None,
            priority: None,
            dependencies: None,
        };

        let task_id = match handler.handle_create_task(create_params).await {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Task(task) => task.id,
                _ => panic!("Expected Task response"),
            },
            _ => panic!("Expected success response"),
        };

        // Delete the task
        let delete_params = DeleteTaskParams { id: task_id };
        let response = handler.handle_delete_task(delete_params).await;

        match response {
            McpResponse::Success(resp) => {
                match resp.result {
                    ResponseResult::Empty => {} // Expected
                    _ => panic!("Expected Empty response"),
                }
            }
            _ => panic!("Expected success response"),
        }

        // Verify task is gone
        let get_params = GetTaskParams { id: task_id };
        let get_response = handler.handle_get_task(get_params).await;

        match get_response {
            McpResponse::Error(_) => {} // Expected
            _ => panic!("Expected error response for deleted task"),
        }
    }

    #[tokio::test]
    async fn test_list_tasks_with_multiple() {
        let handler = create_test_handler().await;

        // Create multiple tasks
        for i in 1..=3 {
            let params = CreateTaskParams {
                title: format!("Task {}", i),
                description: None,
                priority: None,
                dependencies: None,
            };
            handler.handle_create_task(params).await;
        }

        let response = handler.handle_list_tasks().await;

        match response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Tasks(tasks) => {
                    assert_eq!(tasks.len(), 3);
                }
                _ => panic!("Expected Tasks response"),
            },
            _ => panic!("Expected success response"),
        }
    }
}
