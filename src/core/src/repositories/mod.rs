use async_trait::async_trait;
use luce_shared::{Task, TaskId, TaskGraph, LuceError};

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

pub mod task_sqlite;
pub mod graph_sqlite;

pub use task_sqlite::SqliteTaskRepository;
pub use graph_sqlite::SqliteGraphRepository;