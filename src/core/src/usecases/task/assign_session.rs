use crate::repositories::TaskRepository;
use crate::usecases::use_case::UseCase;
use async_trait::async_trait;
use chrono::Utc;
use luce_shared::{LuceError, Task, TaskId};

pub struct AssignSessionInput {
    pub task_id: TaskId,
    pub session_id: Option<String>,
}

impl AssignSessionInput {
    pub fn assign(task_id: TaskId, session_id: String) -> Self {
        Self {
            task_id,
            session_id: Some(session_id),
        }
    }

    pub fn unassign(task_id: TaskId) -> Self {
        Self {
            task_id,
            session_id: None,
        }
    }
}

pub struct AssignSessionUseCase<R: TaskRepository> {
    repository: R,
}

impl<R: TaskRepository> AssignSessionUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: TaskRepository + Send + Sync> UseCase<AssignSessionInput, Task>
    for AssignSessionUseCase<R>
{
    async fn execute(&self, input: AssignSessionInput) -> Result<Task, LuceError> {
        let mut task = self.repository.get_task(input.task_id).await?;
        task.assigned_session = input.session_id;
        task.updated_at = Utc::now();
        self.repository.save_task(&task).await?;
        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::sqlite::SqliteTaskRepository;
    use crate::usecases::task::create_task::{CreateTaskInput, CreateTaskUseCase};
    use tempfile::NamedTempFile;

    async fn create_test_repos() -> (
        CreateTaskUseCase<SqliteTaskRepository>,
        AssignSessionUseCase<SqliteTaskRepository>,
    ) {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().to_str().unwrap());
        let repo1 = SqliteTaskRepository::new(&db_url).await.unwrap();
        let repo2 = SqliteTaskRepository::new(&db_url).await.unwrap();
        (
            CreateTaskUseCase::new(repo1),
            AssignSessionUseCase::new(repo2),
        )
    }

    #[tokio::test]
    async fn test_assign_session() {
        let (create_usecase, assign_usecase) = create_test_repos().await;

        let task = create_usecase
            .execute(CreateTaskInput::new("Test task".to_string()))
            .await
            .unwrap();
        assert_eq!(task.assigned_session, None);

        let input = AssignSessionInput::assign(task.id, "session_123".to_string());
        let assigned_task = assign_usecase.execute(input).await.unwrap();

        assert_eq!(
            assigned_task.assigned_session,
            Some("session_123".to_string())
        );
        assert!(assigned_task.updated_at > task.updated_at);
    }

    #[tokio::test]
    async fn test_unassign_session() {
        let (create_usecase, assign_usecase) = create_test_repos().await;

        let task = create_usecase
            .execute(CreateTaskInput::new("Test task".to_string()))
            .await
            .unwrap();

        // First assign
        let input = AssignSessionInput::assign(task.id, "session_123".to_string());
        let assigned_task = assign_usecase.execute(input).await.unwrap();
        assert_eq!(
            assigned_task.assigned_session,
            Some("session_123".to_string())
        );

        // Then unassign
        let input = AssignSessionInput::unassign(task.id);
        let unassigned_task = assign_usecase.execute(input).await.unwrap();
        assert_eq!(unassigned_task.assigned_session, None);
    }
}
