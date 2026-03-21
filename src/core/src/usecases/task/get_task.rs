use crate::repositories::TaskRepository;
use crate::usecases::use_case::UseCase;
use async_trait::async_trait;
use luce_shared::{LuceError, Task, TaskId};

pub struct GetTaskInput {
    pub task_id: TaskId,
}

impl GetTaskInput {
    pub fn new(task_id: TaskId) -> Self {
        Self { task_id }
    }
}

pub struct GetTaskUseCase<R: TaskRepository> {
    repository: R,
}

impl<R: TaskRepository> GetTaskUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: TaskRepository + Send + Sync> UseCase<GetTaskInput, Task> for GetTaskUseCase<R> {
    async fn execute(&self, input: GetTaskInput) -> Result<Task, LuceError> {
        self.repository.get_task(input.task_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::SqliteTaskRepository;
    use crate::usecases::task::create_task::{CreateTaskInput, CreateTaskUseCase};
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    async fn create_test_repos() -> (
        CreateTaskUseCase<SqliteTaskRepository>,
        GetTaskUseCase<SqliteTaskRepository>,
    ) {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().to_str().unwrap());
        let repo1 = SqliteTaskRepository::new(&db_url).await.unwrap();
        let repo2 = SqliteTaskRepository::new(&db_url).await.unwrap();
        (CreateTaskUseCase::new(repo1), GetTaskUseCase::new(repo2))
    }

    #[tokio::test]
    async fn test_get_existing_task() {
        let (create_usecase, get_usecase) = create_test_repos().await;

        let created_task = create_usecase
            .execute(CreateTaskInput::new("Test task".to_string()))
            .await
            .unwrap();

        let input = GetTaskInput::new(created_task.id);
        let retrieved_task = get_usecase.execute(input).await.unwrap();

        assert_eq!(retrieved_task.id, created_task.id);
        assert_eq!(retrieved_task.title, created_task.title);
    }

    #[tokio::test]
    async fn test_get_nonexistent_task() {
        let (_create_usecase, get_usecase) = create_test_repos().await;

        let nonexistent_id = Uuid::new_v4();
        let input = GetTaskInput::new(nonexistent_id);
        let result = get_usecase.execute(input).await;

        assert!(matches!(result, Err(LuceError::TaskNotFound { id: _ })));
    }
}
