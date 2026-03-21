use async_trait::async_trait;
use luce_shared::{LuceError, Task, TaskDependency, TaskId};
use std::sync::Arc;

#[async_trait]
pub trait TaskRepository {
    async fn save_task(&self, task: &Task) -> Result<(), LuceError>;
    async fn get_task(&self, id: TaskId) -> Result<Task, LuceError>;
    async fn delete_task(&self, id: TaskId) -> Result<(), LuceError>;
    async fn list_tasks(&self) -> Result<Vec<Task>, LuceError>;
}

#[async_trait]
pub trait DependencyRepository {
    async fn save_dependency(&self, dependency: &TaskDependency) -> Result<(), LuceError>;
    async fn remove_dependency(
        &self,
        task_id: TaskId,
        depends_on_task_id: TaskId,
    ) -> Result<(), LuceError>;
    async fn get_dependencies(&self, task_id: TaskId) -> Result<Vec<TaskId>, LuceError>;
    async fn get_dependents(&self, task_id: TaskId) -> Result<Vec<TaskId>, LuceError>;
    async fn remove_all_dependencies(&self, task_id: TaskId) -> Result<(), LuceError>;
}

pub mod dependency_sqlite;
pub mod task_sqlite;

pub use dependency_sqlite::SqliteDependencyRepository;
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

// Implement DependencyRepository for Arc<T> where T: DependencyRepository
#[async_trait]
impl<T: DependencyRepository + Send + Sync> DependencyRepository for Arc<T> {
    async fn save_dependency(&self, dependency: &TaskDependency) -> Result<(), LuceError> {
        self.as_ref().save_dependency(dependency).await
    }

    async fn remove_dependency(
        &self,
        task_id: TaskId,
        depends_on_task_id: TaskId,
    ) -> Result<(), LuceError> {
        self.as_ref()
            .remove_dependency(task_id, depends_on_task_id)
            .await
    }

    async fn get_dependencies(&self, task_id: TaskId) -> Result<Vec<TaskId>, LuceError> {
        self.as_ref().get_dependencies(task_id).await
    }

    async fn get_dependents(&self, task_id: TaskId) -> Result<Vec<TaskId>, LuceError> {
        self.as_ref().get_dependents(task_id).await
    }

    async fn remove_all_dependencies(&self, task_id: TaskId) -> Result<(), LuceError> {
        self.as_ref().remove_all_dependencies(task_id).await
    }
}
