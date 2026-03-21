use async_trait::async_trait;
use luce_shared::{LuceError, Task, TaskGraph, TaskId};
use std::sync::Arc;

#[async_trait]
pub trait TaskRepository {
    async fn save_task(&self, task: &Task) -> Result<(), LuceError>;
    async fn get_task(&self, id: TaskId) -> Result<Task, LuceError>;
    async fn delete_task(&self, id: TaskId) -> Result<(), LuceError>;
    async fn list_tasks(&self) -> Result<Vec<Task>, LuceError>;
}

#[async_trait]
pub trait GraphRepository {
    async fn save_graph(&self, graph: &TaskGraph, id: &str) -> Result<(), LuceError>;
    async fn load_graph(&self, id: &str) -> Result<TaskGraph, LuceError>;
    async fn delete_graph(&self, id: &str) -> Result<(), LuceError>;
    async fn list_graphs(&self) -> Result<Vec<String>, LuceError>;
    async fn graph_exists(&self, id: &str) -> Result<bool, LuceError>;
}

pub mod graph_sqlite;
pub mod task_sqlite;

pub use graph_sqlite::SqliteGraphRepository;
pub use task_sqlite::SqliteTaskRepository;

// Implement TaskRepository for Arc<T> where T: TaskRepository
#[async_trait]
impl<T: TaskRepository + Send + Sync> TaskRepository for Arc<T> {
    async fn save_task(&self, task: &Task) -> Result<(), LuceError> {
        self.as_ref().save_task(task).await
    }

    async fn get_task(&self, id: TaskId) -> Result<Task, LuceError> {
        self.as_ref().get_task(id).await
    }

    async fn delete_task(&self, id: TaskId) -> Result<(), LuceError> {
        self.as_ref().delete_task(id).await
    }

    async fn list_tasks(&self) -> Result<Vec<Task>, LuceError> {
        self.as_ref().list_tasks().await
    }
}

// Implement GraphRepository for Arc<T> where T: GraphRepository
#[async_trait]
impl<T: GraphRepository + Send + Sync> GraphRepository for Arc<T> {
    async fn save_graph(&self, graph: &TaskGraph, id: &str) -> Result<(), LuceError> {
        self.as_ref().save_graph(graph, id).await
    }

    async fn load_graph(&self, id: &str) -> Result<TaskGraph, LuceError> {
        self.as_ref().load_graph(id).await
    }

    async fn delete_graph(&self, id: &str) -> Result<(), LuceError> {
        self.as_ref().delete_graph(id).await
    }

    async fn list_graphs(&self) -> Result<Vec<String>, LuceError> {
        self.as_ref().list_graphs().await
    }

    async fn graph_exists(&self, id: &str) -> Result<bool, LuceError> {
        self.as_ref().graph_exists(id).await
    }
}
