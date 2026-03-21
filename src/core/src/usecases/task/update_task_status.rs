use crate::repositories::TaskRepository;
use crate::usecases::use_case::UseCase;
use async_trait::async_trait;
use chrono::Utc;
use luce_shared::{LuceError, Task, TaskId, TaskStatus};

pub struct UpdateTaskStatusInput {
    pub task_id: TaskId,
    pub new_status: TaskStatus,
}

impl UpdateTaskStatusInput {
    pub fn new(task_id: TaskId, new_status: TaskStatus) -> Self {
        Self {
            task_id,
            new_status,
        }
    }
}

pub struct UpdateTaskStatusUseCase<R: TaskRepository> {
    repository: R,
}

impl<R: TaskRepository> UpdateTaskStatusUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    fn validate_status_transition(
        &self,
        current: &TaskStatus,
        new: &TaskStatus,
    ) -> Result<(), LuceError> {
        use TaskStatus::*;

        let valid = match (current, new) {
            // From Pending
            (Pending, Ready) | (Pending, Blocked) => true,
            // From Ready
            (Ready, InProgress) | (Ready, Blocked) => true,
            // From InProgress
            (InProgress, Completed) | (InProgress, Failed) | (InProgress, Ready) => true,
            // From Completed (generally final, but allow reopening)
            (Completed, Ready) | (Completed, InProgress) => true,
            // From Failed (allow retry)
            (Failed, Ready) | (Failed, InProgress) => true,
            // From Blocked
            (Blocked, Ready) | (Blocked, Pending) => true,
            // Same status is always valid
            (a, b) if a == b => true,
            // All other transitions are invalid
            _ => false,
        };

        if !valid {
            return Err(LuceError::InvalidStateTransition {
                from: format!("{:?}", current),
                to: format!("{:?}", new),
            });
        }

        Ok(())
    }
}

#[async_trait]
impl<R: TaskRepository + Send + Sync> UseCase<UpdateTaskStatusInput, Task>
    for UpdateTaskStatusUseCase<R>
{
    async fn execute(&self, input: UpdateTaskStatusInput) -> Result<Task, LuceError> {
        let mut task = self.repository.get_task(input.task_id).await?;

        // Validate state transition
        self.validate_status_transition(&task.status, &input.new_status)?;

        let new_status = input.new_status;
        task.status = new_status;
        task.updated_at = Utc::now();

        // Update timestamps based on status
        match new_status {
            TaskStatus::InProgress => {
                if task.started_at.is_none() {
                    task.started_at = Some(Utc::now());
                }
            }
            TaskStatus::Completed | TaskStatus::Failed => {
                if task.completed_at.is_none() {
                    task.completed_at = Some(Utc::now());
                }
            }
            _ => {}
        }

        self.repository.save_task(&task).await?;
        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::SqliteTaskRepository;
    use crate::usecases::task::create_task::{CreateTaskInput, CreateTaskUseCase};
    use tempfile::NamedTempFile;

    async fn create_test_repos() -> (
        CreateTaskUseCase<SqliteTaskRepository>,
        UpdateTaskStatusUseCase<SqliteTaskRepository>,
    ) {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().to_str().unwrap());
        let repo1 = SqliteTaskRepository::new(&db_url).await.unwrap();
        let repo2 = SqliteTaskRepository::new(&db_url).await.unwrap();
        (
            CreateTaskUseCase::new(repo1),
            UpdateTaskStatusUseCase::new(repo2),
        )
    }

    #[tokio::test]
    async fn test_update_task_status_valid_transition() {
        let (create_usecase, update_usecase) = create_test_repos().await;

        let task = create_usecase
            .execute(CreateTaskInput::new("Test task".to_string()))
            .await
            .unwrap();

        let input = UpdateTaskStatusInput::new(task.id, TaskStatus::Ready);
        let updated_task = update_usecase.execute(input).await.unwrap();

        assert_eq!(updated_task.status, TaskStatus::Ready);
        assert!(updated_task.updated_at > task.updated_at);
    }

    #[tokio::test]
    async fn test_update_task_status_with_timestamps() {
        let (create_usecase, update_usecase) = create_test_repos().await;

        let task = create_usecase
            .execute(CreateTaskInput::new("Test task".to_string()))
            .await
            .unwrap();

        // Move to InProgress
        let input = UpdateTaskStatusInput::new(task.id, TaskStatus::Ready);
        update_usecase.execute(input).await.unwrap();

        let input = UpdateTaskStatusInput::new(task.id, TaskStatus::InProgress);
        let in_progress_task = update_usecase.execute(input).await.unwrap();

        assert_eq!(in_progress_task.status, TaskStatus::InProgress);
        assert!(in_progress_task.started_at.is_some());

        // Complete the task
        let input = UpdateTaskStatusInput::new(task.id, TaskStatus::Completed);
        let completed_task = update_usecase.execute(input).await.unwrap();

        assert_eq!(completed_task.status, TaskStatus::Completed);
        assert!(completed_task.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_invalid_status_transition() {
        let (create_usecase, update_usecase) = create_test_repos().await;

        let task = create_usecase
            .execute(CreateTaskInput::new("Test task".to_string()))
            .await
            .unwrap();

        // Try invalid transition: Pending -> Completed
        let input = UpdateTaskStatusInput::new(task.id, TaskStatus::Completed);
        let result = update_usecase.execute(input).await;

        assert!(matches!(
            result,
            Err(LuceError::InvalidStateTransition { .. })
        ));
    }
}
