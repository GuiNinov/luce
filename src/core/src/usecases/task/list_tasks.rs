use async_trait::async_trait;
use luce_shared::{Task, TaskStatus, LuceError};
use crate::repositories::TaskRepository;
use crate::usecases::use_case::UseCase;

#[derive(Clone)]
pub enum TaskFilter {
    All,
    ByStatus(TaskStatus),
    BySession(String),
    Unassigned,
}

pub struct ListTasksInput {
    pub filter: TaskFilter,
}

impl ListTasksInput {
    pub fn all() -> Self {
        Self {
            filter: TaskFilter::All,
        }
    }

    pub fn by_status(status: TaskStatus) -> Self {
        Self {
            filter: TaskFilter::ByStatus(status),
        }
    }

    pub fn by_session(session_id: String) -> Self {
        Self {
            filter: TaskFilter::BySession(session_id),
        }
    }

    pub fn unassigned() -> Self {
        Self {
            filter: TaskFilter::Unassigned,
        }
    }
}

pub struct ListTasksUseCase<R: TaskRepository> {
    repository: R,
}

impl<R: TaskRepository> ListTasksUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    fn apply_filter(&self, tasks: Vec<Task>, filter: &TaskFilter) -> Vec<Task> {
        match filter {
            TaskFilter::All => tasks,
            TaskFilter::ByStatus(status) => {
                tasks.into_iter().filter(|task| task.status == *status).collect()
            }
            TaskFilter::BySession(session_id) => {
                tasks
                    .into_iter()
                    .filter(|task| task.assigned_session.as_deref() == Some(session_id))
                    .collect()
            }
            TaskFilter::Unassigned => {
                tasks
                    .into_iter()
                    .filter(|task| task.assigned_session.is_none())
                    .collect()
            }
        }
    }
}

#[async_trait]
impl<R: TaskRepository + Send + Sync> UseCase<ListTasksInput, Vec<Task>> for ListTasksUseCase<R> {
    async fn execute(&self, input: ListTasksInput) -> Result<Vec<Task>, LuceError> {
        let tasks = self.repository.list_tasks().await?;
        Ok(self.apply_filter(tasks, &input.filter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::sqlite::SqliteTaskRepository;
    use crate::usecases::task::create_task::{CreateTaskUseCase, CreateTaskInput};
    use crate::usecases::task::update_task_status::{UpdateTaskStatusUseCase, UpdateTaskStatusInput};
    use crate::usecases::task::assign_session::{AssignSessionUseCase, AssignSessionInput};
    use tempfile::NamedTempFile;

    async fn create_test_repos() -> (
        CreateTaskUseCase<SqliteTaskRepository>,
        UpdateTaskStatusUseCase<SqliteTaskRepository>,
        AssignSessionUseCase<SqliteTaskRepository>,
        ListTasksUseCase<SqliteTaskRepository>,
    ) {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().to_str().unwrap());
        let repo1 = SqliteTaskRepository::new(&db_url).await.unwrap();
        let repo2 = SqliteTaskRepository::new(&db_url).await.unwrap();
        let repo3 = SqliteTaskRepository::new(&db_url).await.unwrap();
        let repo4 = SqliteTaskRepository::new(&db_url).await.unwrap();
        (
            CreateTaskUseCase::new(repo1),
            UpdateTaskStatusUseCase::new(repo2),
            AssignSessionUseCase::new(repo3),
            ListTasksUseCase::new(repo4),
        )
    }

    #[tokio::test]
    async fn test_list_all_tasks() {
        let (create_usecase, _, _, list_usecase) = create_test_repos().await;
        
        create_usecase.execute(CreateTaskInput::new("Task 1".to_string())).await.unwrap();
        create_usecase.execute(CreateTaskInput::new("Task 2".to_string())).await.unwrap();
        
        let input = ListTasksInput::all();
        let tasks = list_usecase.execute(input).await.unwrap();
        
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_list_tasks_by_status() {
        let (create_usecase, update_usecase, _, list_usecase) = create_test_repos().await;
        
        let task1 = create_usecase.execute(CreateTaskInput::new("Task 1".to_string())).await.unwrap();
        let task2 = create_usecase.execute(CreateTaskInput::new("Task 2".to_string())).await.unwrap();
        
        update_usecase.execute(UpdateTaskStatusInput::new(task2.id, TaskStatus::Ready)).await.unwrap();
        
        let pending_input = ListTasksInput::by_status(TaskStatus::Pending);
        let pending_tasks = list_usecase.execute(pending_input).await.unwrap();
        
        let ready_input = ListTasksInput::by_status(TaskStatus::Ready);
        let ready_tasks = list_usecase.execute(ready_input).await.unwrap();
        
        assert_eq!(pending_tasks.len(), 1);
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(pending_tasks[0].id, task1.id);
        assert_eq!(ready_tasks[0].id, task2.id);
    }

    #[tokio::test]
    async fn test_list_tasks_by_session() {
        let (create_usecase, _, assign_usecase, list_usecase) = create_test_repos().await;
        
        let task1 = create_usecase.execute(CreateTaskInput::new("Task 1".to_string())).await.unwrap();
        let task2 = create_usecase.execute(CreateTaskInput::new("Task 2".to_string())).await.unwrap();
        
        assign_usecase.execute(AssignSessionInput::assign(task1.id, "session_123".to_string())).await.unwrap();
        
        let session_input = ListTasksInput::by_session("session_123".to_string());
        let session_tasks = list_usecase.execute(session_input).await.unwrap();
        
        let unassigned_input = ListTasksInput::unassigned();
        let unassigned_tasks = list_usecase.execute(unassigned_input).await.unwrap();
        
        assert_eq!(session_tasks.len(), 1);
        assert_eq!(unassigned_tasks.len(), 1);
        assert_eq!(session_tasks[0].id, task1.id);
        assert_eq!(unassigned_tasks[0].id, task2.id);
    }
}