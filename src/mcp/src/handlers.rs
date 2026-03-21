use crate::protocol::{
    CreateTaskParams, DeleteTaskParams, ErrorResponse, GetTaskParams, McpError, McpRequest,
    McpResponse, ResponseResult, SuccessResponse, UpdateTaskParams,
};
use luce_shared::{LuceError, Task, TaskGraph};

pub struct TaskHandler {
    graph: TaskGraph,
}

impl TaskHandler {
    pub fn new() -> Self {
        Self {
            graph: TaskGraph::new(),
        }
    }

    pub fn handle_request(&mut self, request: McpRequest) -> McpResponse {
        match request {
            McpRequest::ListTasks => self.handle_list_tasks(),
            McpRequest::CreateTask { params } => self.handle_create_task(params),
            McpRequest::GetTask { params } => self.handle_get_task(params),
            McpRequest::UpdateTask { params } => self.handle_update_task(params),
            McpRequest::DeleteTask { params } => self.handle_delete_task(params),
            McpRequest::GetGraph => self.handle_get_graph(),
            McpRequest::GetReadyTasks => self.handle_get_ready_tasks(),
        }
    }

    fn handle_list_tasks(&self) -> McpResponse {
        let tasks = self.graph.tasks.values().cloned().collect::<Vec<_>>();
        McpResponse::Success(Box::new(SuccessResponse {
            result: ResponseResult::Tasks(tasks),
        }))
    }

    fn handle_create_task(&mut self, params: CreateTaskParams) -> McpResponse {
        let mut task = Task::new(params.title);

        if let Some(desc) = params.description {
            task = task.with_description(desc);
        }

        if let Some(priority) = params.priority {
            task = task.with_priority(priority);
        }

        let task_id = task.id;

        // Add task to graph (add_task returns TaskId, not Result)
        self.graph.add_task(task.clone());

        // Add dependencies if provided
        if let Some(deps) = params.dependencies {
            for dep_id in deps {
                if let Err(e) = self.graph.add_dependency(task_id, dep_id) {
                    return self.handle_error(e);
                }
            }
        }

        // Get the updated task from the graph
        match self.graph.get_task(&task_id) {
            Some(task) => McpResponse::Success(Box::new(SuccessResponse {
                result: ResponseResult::Task(Box::new(task.clone())),
            })),
            None => McpResponse::Error(ErrorResponse {
                error: McpError::internal_error(),
            }),
        }
    }

    fn handle_get_task(&self, params: GetTaskParams) -> McpResponse {
        match self.graph.get_task(&params.id) {
            Some(task) => McpResponse::Success(Box::new(SuccessResponse {
                result: ResponseResult::Task(Box::new(task.clone())),
            })),
            None => McpResponse::Error(ErrorResponse {
                error: McpError::task_not_found(params.id),
            }),
        }
    }

    fn handle_update_task(&mut self, params: UpdateTaskParams) -> McpResponse {
        let task = match self.graph.get_task_mut(&params.id) {
            Some(task) => task,
            None => {
                return McpResponse::Error(ErrorResponse {
                    error: McpError::task_not_found(params.id),
                })
            }
        };

        if let Some(title) = params.title {
            task.set_title(title);
        }

        if let Some(description) = params.description {
            task.set_description(Some(description));
        }

        if let Some(status) = params.status {
            task.set_status(status);
        }

        if let Some(priority) = params.priority {
            task.set_priority(priority);
        }

        McpResponse::Success(Box::new(SuccessResponse {
            result: ResponseResult::Task(Box::new(task.clone())),
        }))
    }

    fn handle_delete_task(&mut self, params: DeleteTaskParams) -> McpResponse {
        match self.graph.remove_task(&params.id) {
            Some(_) => McpResponse::Success(Box::new(SuccessResponse {
                result: ResponseResult::Empty,
            })),
            None => McpResponse::Error(ErrorResponse {
                error: McpError::task_not_found(params.id),
            }),
        }
    }

    fn handle_get_graph(&self) -> McpResponse {
        McpResponse::Success(Box::new(SuccessResponse {
            result: ResponseResult::Graph(self.graph.clone()),
        }))
    }

    fn handle_get_ready_tasks(&self) -> McpResponse {
        let ready_tasks = self
            .graph
            .get_ready_tasks()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        McpResponse::Success(Box::new(SuccessResponse {
            result: ResponseResult::Tasks(ready_tasks),
        }))
    }

    fn handle_error(&self, error: LuceError) -> McpResponse {
        let mcp_error = match error {
            LuceError::TaskNotFound { id } => McpError::task_not_found(id),
            LuceError::CircularDependency => McpError::dependency_cycle(),
            LuceError::DependencyError { message } => McpError::invalid_params(message),
            LuceError::InvalidStateTransition { from, to } => McpError::invalid_params(format!(
                "Invalid state transition from {} to {}",
                from, to
            )),
            LuceError::SerializationError(_) => McpError::internal_error(),
            LuceError::IoError(_) => McpError::internal_error(),
        };

        McpResponse::Error(ErrorResponse { error: mcp_error })
    }
}

impl Default for TaskHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luce_shared::{TaskPriority, TaskStatus};
    use uuid::Uuid;

    #[test]
    fn test_handler_creation() {
        let handler = TaskHandler::new();
        assert_eq!(handler.graph.task_count(), 0);
    }

    #[test]
    fn test_list_tasks_empty() {
        let handler = TaskHandler::new();
        let response = handler.handle_list_tasks();

        match response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Tasks(tasks) => assert!(tasks.is_empty()),
                _ => panic!("Expected Tasks response"),
            },
            _ => panic!("Expected success response"),
        }
    }

    #[test]
    fn test_create_task_basic() {
        let mut handler = TaskHandler::new();
        let params = CreateTaskParams {
            title: "Test task".to_string(),
            description: None,
            priority: None,
            dependencies: None,
        };

        let response = handler.handle_create_task(params);

        match response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Task(task) => {
                    assert_eq!(task.title, "Test task");
                    assert_eq!(task.priority, TaskPriority::Normal);
                    assert!(task.description.is_none());
                    assert!(task.dependencies.is_empty());
                }
                _ => panic!("Expected Task response"),
            },
            _ => panic!("Expected success response"),
        }
    }

    #[test]
    fn test_create_task_with_options() {
        let mut handler = TaskHandler::new();
        let params = CreateTaskParams {
            title: "Test task".to_string(),
            description: Some("A test description".to_string()),
            priority: Some(TaskPriority::High),
            dependencies: None,
        };

        let response = handler.handle_create_task(params);

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

    #[test]
    fn test_get_task_exists() {
        let mut handler = TaskHandler::new();

        // First create a task
        let create_params = CreateTaskParams {
            title: "Test task".to_string(),
            description: None,
            priority: None,
            dependencies: None,
        };

        let create_response = handler.handle_create_task(create_params);
        let task_id = match create_response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Task(task) => task.id,
                _ => panic!("Expected Task response"),
            },
            _ => panic!("Expected success response"),
        };

        // Now get the task
        let get_params = GetTaskParams { id: task_id };
        let response = handler.handle_get_task(get_params);

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

    #[test]
    fn test_get_task_not_found() {
        let handler = TaskHandler::new();
        let nonexistent_id = Uuid::new_v4();
        let params = GetTaskParams { id: nonexistent_id };

        let response = handler.handle_get_task(params);

        match response {
            McpResponse::Error(err) => {
                assert_eq!(err.error.code, 1001);
                assert!(err.error.message.contains("Task not found"));
            }
            _ => panic!("Expected error response"),
        }
    }

    #[test]
    fn test_update_task() {
        let mut handler = TaskHandler::new();

        // Create a task first
        let create_params = CreateTaskParams {
            title: "Original title".to_string(),
            description: None,
            priority: None,
            dependencies: None,
        };

        let task_id = match handler.handle_create_task(create_params) {
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

        let response = handler.handle_update_task(update_params);

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

    #[test]
    fn test_update_task_not_found() {
        let mut handler = TaskHandler::new();
        let nonexistent_id = Uuid::new_v4();

        let params = UpdateTaskParams {
            id: nonexistent_id,
            title: Some("Updated title".to_string()),
            description: None,
            status: None,
            priority: None,
        };

        let response = handler.handle_update_task(params);

        match response {
            McpResponse::Error(err) => {
                assert_eq!(err.error.code, 1001);
            }
            _ => panic!("Expected error response"),
        }
    }

    #[test]
    fn test_delete_task() {
        let mut handler = TaskHandler::new();

        // Create a task first
        let create_params = CreateTaskParams {
            title: "To be deleted".to_string(),
            description: None,
            priority: None,
            dependencies: None,
        };

        let task_id = match handler.handle_create_task(create_params) {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Task(task) => task.id,
                _ => panic!("Expected Task response"),
            },
            _ => panic!("Expected success response"),
        };

        // Delete the task
        let delete_params = DeleteTaskParams { id: task_id };
        let response = handler.handle_delete_task(delete_params);

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
        let get_response = handler.handle_get_task(get_params);

        match get_response {
            McpResponse::Error(_) => {} // Expected
            _ => panic!("Expected error response for deleted task"),
        }
    }

    #[test]
    fn test_get_graph() {
        let mut handler = TaskHandler::new();

        // Add some tasks
        let create_params = CreateTaskParams {
            title: "Task 1".to_string(),
            description: None,
            priority: None,
            dependencies: None,
        };
        handler.handle_create_task(create_params);

        let response = handler.handle_get_graph();

        match response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Graph(graph) => {
                    assert_eq!(graph.task_count(), 1);
                }
                _ => panic!("Expected Graph response"),
            },
            _ => panic!("Expected success response"),
        }
    }

    #[test]
    fn test_get_ready_tasks() {
        let mut handler = TaskHandler::new();

        // Create some tasks with different statuses
        let create_params1 = CreateTaskParams {
            title: "Ready task".to_string(),
            description: None,
            priority: None,
            dependencies: None,
        };
        let task1_id = match handler.handle_create_task(create_params1) {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Task(task) => task.id,
                _ => panic!("Expected Task response"),
            },
            _ => panic!("Expected success response"),
        };

        // Set the task to ready
        let update_params = UpdateTaskParams {
            id: task1_id,
            title: None,
            description: None,
            status: Some(TaskStatus::Ready),
            priority: None,
        };
        handler.handle_update_task(update_params);

        let response = handler.handle_get_ready_tasks();

        match response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Tasks(tasks) => {
                    assert_eq!(tasks.len(), 1);
                    assert_eq!(tasks[0].status, TaskStatus::Ready);
                }
                _ => panic!("Expected Tasks response"),
            },
            _ => panic!("Expected success response"),
        }
    }

    #[test]
    fn test_list_tasks_with_multiple() {
        let mut handler = TaskHandler::new();

        // Create multiple tasks
        for i in 1..=3 {
            let params = CreateTaskParams {
                title: format!("Task {}", i),
                description: None,
                priority: None,
                dependencies: None,
            };
            handler.handle_create_task(params);
        }

        let response = handler.handle_list_tasks();

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
