use crate::protocol::{
    CreateTaskParams, DeleteTaskParams, ErrorResponse, GetTaskParams, McpError, McpRequest,
    McpResponse, ResponseResult, SuccessResponse, UpdateTaskParams,
};
use luce_core::{
    CreateTaskInput, CreateTaskUseCase, GetTaskInput, GetTaskUseCase, ListTasksInput,
    ListTasksUseCase, SqliteTaskRepository, TaskRepository, UpdateTaskStatusInput,
    UpdateTaskStatusUseCase, UseCase,
};
use luce_shared::{LuceError, Task, TaskStatus};
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
        }
    }

    async fn handle_list_tasks(&self) -> McpResponse {
        let usecase = ListTasksUseCase::new(Arc::clone(&self.task_repository));
        let input = ListTasksInput::default();

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
            Ok(task) => {
                // Add task to the graph
                let add_task_usecase = luce_core::AddTaskToGraphUseCase::new(
                    Arc::clone(&self.task_repository),
                    Arc::clone(&self.graph_repository),
                );
                let add_input = luce_core::AddTaskToGraphInput {
                    graph_id: self.current_graph_id.clone(),
                    task_id: task.id,
                };

                if let Err(e) = add_task_usecase.execute(add_input).await {
                    return self.handle_error(e);
                }

                // Add dependencies if provided
                if let Some(deps) = params.dependencies {
                    for dep_id in deps {
                        let add_dep_usecase = luce_core::AddDependencyUseCase::new(
                            Arc::clone(&self.task_repository),
                            Arc::clone(&self.graph_repository),
                        );
                        let dep_input = luce_core::AddDependencyInput {
                            graph_id: self.current_graph_id.clone(),
                            task_id: task.id,
                            dependency_id: dep_id,
                        };

                        if let Err(e) = add_dep_usecase.execute(dep_input).await {
                            return self.handle_error(e);
                        }
                    }
                }

                McpResponse::Success(Box::new(SuccessResponse {
                    result: ResponseResult::Task(Box::new(task)),
                }))
            }
            Err(e) => self.handle_error(e),
        }
    }

    async fn handle_get_task(&self, params: GetTaskParams) -> McpResponse {
        let usecase = GetTaskUseCase::new(Arc::clone(&self.task_repository));
        let input = GetTaskInput { id: params.id };

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
        let get_input = GetTaskInput { id: params.id };

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
        // Remove task from graph first
        let remove_usecase = luce_core::RemoveTaskFromGraphUseCase::new(
            Arc::clone(&self.task_repository),
            Arc::clone(&self.graph_repository),
        );
        let remove_input = luce_core::RemoveTaskFromGraphInput {
            graph_id: self.current_graph_id.clone(),
            task_id: params.id,
        };

        if let Err(e) = remove_usecase.execute(remove_input).await {
            return self.handle_error(e);
        }

        // Then delete the task from repository
        match self.task_repository.delete_task(params.id).await {
            Ok(_) => McpResponse::Success(Box::new(SuccessResponse {
                result: ResponseResult::Empty,
            })),
            Err(e) => self.handle_error(e),
        }
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
    async fn test_get_graph() {
        let handler = create_test_handler().await;

        let response = handler.handle_get_graph().await;

        match response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Graph(_graph) => {
                    // Just verify we can get the graph
                }
                _ => panic!("Expected Graph response"),
            },
            _ => panic!("Expected success response"),
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
