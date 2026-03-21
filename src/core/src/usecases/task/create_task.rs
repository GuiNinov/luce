use crate::repositories::TaskRepository;
use crate::usecases::use_case::UseCase;
use async_trait::async_trait;
use luce_shared::{LuceError, Task, TaskPriority};

pub struct CreateTaskInput {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,
}

impl CreateTaskInput {
    pub fn new(title: String) -> Self {
        Self {
            title,
            description: None,
            priority: None,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = Some(priority);
        self
    }
}

pub struct CreateTaskUseCase<R: TaskRepository> {
    repository: R,
}

impl<R: TaskRepository> CreateTaskUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: TaskRepository + Send + Sync> UseCase<CreateTaskInput, Task> for CreateTaskUseCase<R> {
    async fn execute(&self, input: CreateTaskInput) -> Result<Task, LuceError> {
        let mut task = Task::new(input.title);

        if let Some(description) = input.description {
            task = task.with_description(description);
        }

        if let Some(priority) = input.priority {
            task = task.with_priority(priority);
        }

        self.repository.save_task(&task).await?;
        Ok(task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::SqliteTaskRepository;
    use tempfile::NamedTempFile;

    async fn create_test_usecase() -> CreateTaskUseCase<SqliteTaskRepository> {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().to_str().unwrap());
        let repo = SqliteTaskRepository::new(&db_url).await.unwrap();
        CreateTaskUseCase::new(repo)
    }

    #[tokio::test]
    async fn test_create_simple_task() {
        let usecase = create_test_usecase().await;
        let input = CreateTaskInput::new("Test task".to_string());

        let task = usecase.execute(input).await.unwrap();

        assert_eq!(task.title, "Test task");
        assert_eq!(task.description, None);
        assert_eq!(task.priority, TaskPriority::Normal);
    }

    #[tokio::test]
    async fn test_create_task_with_details() {
        let usecase = create_test_usecase().await;
        let input = CreateTaskInput::new("Important task".to_string())
            .with_description("This is very important".to_string())
            .with_priority(TaskPriority::High);

        let task = usecase.execute(input).await.unwrap();

        assert_eq!(task.title, "Important task");
        assert_eq!(task.description, Some("This is very important".to_string()));
        assert_eq!(task.priority, TaskPriority::High);
    }
}
