use crate::handlers::TaskHandler;
use crate::protocol::{McpRequest, McpResponse};
use anyhow::Result;
use luce_shared::LuceError;
use std::io::{self, BufRead, Write};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;

pub struct McpServer {
    handler: Mutex<TaskHandler>,
}

impl McpServer {
    pub async fn new(db_path: &str) -> Result<Self, LuceError> {
        let handler = TaskHandler::new(db_path).await?;
        Ok(Self {
            handler: Mutex::new(handler),
        })
    }

    pub async fn run_stdio(&self) -> Result<()> {
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;

            if bytes_read == 0 {
                break;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let response = match serde_json::from_str::<McpRequest>(trimmed) {
                Ok(request) => {
                    let handler = self.handler.lock().await;
                    handler.handle_request(request).await
                }
                Err(_) => McpResponse::Error(crate::protocol::ErrorResponse {
                    error: crate::protocol::McpError::parse_error(),
                }),
            };

            let response_json = serde_json::to_string(&response)?;
            stdout.write_all(response_json.as_bytes()).await?;
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        Ok(())
    }

    pub fn run_sync(&self) -> Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();
        let rt = tokio::runtime::Runtime::new()?;

        for line in stdin.lock().lines() {
            let line = line?;
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            let response = match serde_json::from_str::<McpRequest>(trimmed) {
                Ok(request) => rt.block_on(async {
                    let handler = self.handler.lock().await;
                    handler.handle_request(request).await
                }),
                Err(_) => McpResponse::Error(crate::protocol::ErrorResponse {
                    error: crate::protocol::McpError::parse_error(),
                }),
            };

            let response_json = serde_json::to_string(&response)?;
            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{CreateTaskParams, GetTaskParams, ResponseResult};
    use luce_shared::TaskPriority;
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    async fn create_test_server() -> McpServer {
        let temp_file = NamedTempFile::new().unwrap();
        let db_path = temp_file.path().to_str().unwrap();
        McpServer::new(db_path).await.unwrap()
    }

    #[tokio::test]
    async fn test_server_creation() {
        let _server = create_test_server().await;
        // Just verify it can be created without panicking
    }

    #[tokio::test]
    async fn test_request_handling_through_server() {
        let server = create_test_server().await;

        // Create a task through the server
        let create_request = McpRequest::CreateTask {
            params: CreateTaskParams {
                title: "Server Test Task".to_string(),
                description: Some("Testing server functionality".to_string()),
                priority: Some(TaskPriority::High),
                dependencies: None,
            },
        };

        let handler = server.handler.lock().await;
        let response = handler.handle_request(create_request).await;

        match response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Task(task) => {
                    assert_eq!(task.title, "Server Test Task");
                    assert_eq!(
                        task.description,
                        Some("Testing server functionality".to_string())
                    );
                    assert_eq!(task.priority, TaskPriority::High);
                }
                _ => panic!("Expected Task response"),
            },
            _ => panic!("Expected success response"),
        }
    }

    #[tokio::test]
    async fn test_list_tasks_through_server() {
        let server = create_test_server().await;
        let handler = server.handler.lock().await;

        // Initially empty
        let list_request = McpRequest::ListTasks;
        let response = handler.handle_request(list_request).await;

        match response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Tasks(tasks) => {
                    assert!(tasks.is_empty());
                }
                _ => panic!("Expected Tasks response"),
            },
            _ => panic!("Expected success response"),
        }

        // Add a task and list again
        let create_request = McpRequest::CreateTask {
            params: CreateTaskParams {
                title: "Task for listing".to_string(),
                description: None,
                priority: None,
                dependencies: None,
            },
        };
        handler.handle_request(create_request).await;

        let list_request = McpRequest::ListTasks;
        let response = handler.handle_request(list_request).await;

        match response {
            McpResponse::Success(resp) => match resp.result {
                ResponseResult::Tasks(tasks) => {
                    assert_eq!(tasks.len(), 1);
                    assert_eq!(tasks[0].title, "Task for listing");
                }
                _ => panic!("Expected Tasks response"),
            },
            _ => panic!("Expected success response"),
        }
    }

    #[tokio::test]
    async fn test_error_handling_through_server() {
        let server = create_test_server().await;
        let handler = server.handler.lock().await;

        // Try to get a non-existent task
        let nonexistent_id = Uuid::new_v4();
        let get_request = McpRequest::GetTask {
            params: GetTaskParams { id: nonexistent_id },
        };

        let response = handler.handle_request(get_request).await;

        match response {
            McpResponse::Error(err) => {
                assert_eq!(err.error.code, 1001);
                assert!(err.error.message.contains("Task not found"));
            }
            _ => panic!("Expected error response"),
        }
    }
}
