use async_trait::async_trait;
use luce_shared::{
    CredentialData, CredentialId, CreateCredentialInput, IntegrationCredential, IntegrationType,
    LuceError, Task, TaskDependency, TaskId, UpdateCredentialInput,
};
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

#[async_trait]
pub trait CredentialRepository {
    async fn create_credential(&self, input: CreateCredentialInput) -> Result<IntegrationCredential, LuceError>;
    async fn get_credential(&self, id: CredentialId) -> Result<IntegrationCredential, LuceError>;
    async fn list_credentials(&self, integration_type: Option<IntegrationType>, active_only: bool) -> Result<Vec<IntegrationCredential>, LuceError>;
    async fn update_credential(&self, id: CredentialId, input: UpdateCredentialInput) -> Result<IntegrationCredential, LuceError>;
    async fn delete_credential(&self, id: CredentialId) -> Result<(), LuceError>;
    async fn mark_credential_used(&self, id: CredentialId) -> Result<(), LuceError>;
    async fn get_credential_data(&self, id: CredentialId) -> Result<CredentialData, LuceError>;
}

pub mod credentials_sqlite;
pub mod dependency_sqlite;
pub mod task_sqlite;

pub use credentials_sqlite::SqliteCredentialRepository;
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

// Implement CredentialRepository for Arc<T> where T: CredentialRepository
#[async_trait]
impl<T: CredentialRepository + Send + Sync> CredentialRepository for Arc<T> {
    async fn create_credential(&self, input: CreateCredentialInput) -> Result<IntegrationCredential, LuceError> {
        self.as_ref().create_credential(input).await
    }

    async fn get_credential(&self, id: CredentialId) -> Result<IntegrationCredential, LuceError> {
        self.as_ref().get_credential(id).await
    }

    async fn list_credentials(&self, integration_type: Option<IntegrationType>, active_only: bool) -> Result<Vec<IntegrationCredential>, LuceError> {
        self.as_ref().list_credentials(integration_type, active_only).await
    }

    async fn update_credential(&self, id: CredentialId, input: UpdateCredentialInput) -> Result<IntegrationCredential, LuceError> {
        self.as_ref().update_credential(id, input).await
    }

    async fn delete_credential(&self, id: CredentialId) -> Result<(), LuceError> {
        self.as_ref().delete_credential(id).await
    }

    async fn mark_credential_used(&self, id: CredentialId) -> Result<(), LuceError> {
        self.as_ref().mark_credential_used(id).await
    }

    async fn get_credential_data(&self, id: CredentialId) -> Result<CredentialData, LuceError> {
        self.as_ref().get_credential_data(id).await
    }
}
