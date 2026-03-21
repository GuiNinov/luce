use luce_core::{
    AssignSessionInput, AssignSessionUseCase, CreateTaskInput, CreateTaskUseCase, GetTaskInput,
    GetTaskUseCase, ListTasksInput, ListTasksUseCase, SqliteGraphRepository, SqliteTaskRepository,
    TaskFilter, UpdateTaskStatusInput, UpdateTaskStatusUseCase, UseCase,
};
use luce_shared::{LuceError, Task, TaskId, TaskPriority, TaskStatus};
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

pub struct LuceService {
    task_repository: SqliteTaskRepository,
    graph_repository: SqliteGraphRepository,
}

impl LuceService {
    pub async fn new() -> Result<Self, LuceError> {
        Self::with_database("luce.db").await
    }

    pub async fn with_database<P: AsRef<Path>>(db_path: P) -> Result<Self, LuceError> {
        let db_url = format!("sqlite:{}", db_path.as_ref().to_str().unwrap());

        let task_repository = SqliteTaskRepository::new(&db_url).await?;
        let graph_repository = SqliteGraphRepository::new(&db_url).await?;

        Ok(Self {
            task_repository,
            graph_repository,
        })
    }

    pub async fn create_task(
        &self,
        title: String,
        description: Option<String>,
        priority: Option<TaskPriority>,
    ) -> Result<Task, LuceError> {
        let usecase = CreateTaskUseCase::new(&self.task_repository);
        let mut input = CreateTaskInput::new(title);

        if let Some(desc) = description {
            input = input.with_description(desc);
        }

        if let Some(prio) = priority {
            input = input.with_priority(prio);
        }

        usecase.execute(input).await
    }

    pub async fn get_task(&self, task_id: &str) -> Result<Task, LuceError> {
        let uuid =
            Uuid::from_str(task_id).map_err(|_| LuceError::InvalidTaskId(task_id.to_string()))?;
        let task_id = TaskId::from(uuid);

        let usecase = GetTaskUseCase::new(&self.task_repository);
        let input = GetTaskInput::new(task_id);

        usecase.execute(input).await
    }

    pub async fn list_tasks(&self, filter: Option<TaskFilter>) -> Result<Vec<Task>, LuceError> {
        let usecase = ListTasksUseCase::new(&self.task_repository);
        let input = match filter {
            Some(TaskFilter::ByStatus(status)) => ListTasksInput::by_status(status),
            Some(TaskFilter::BySession(session_id)) => ListTasksInput::by_session(session_id),
            Some(TaskFilter::Unassigned) => ListTasksInput::unassigned(),
            Some(TaskFilter::All) | None => ListTasksInput::all(),
        };

        usecase.execute(input).await
    }

    pub async fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
    ) -> Result<(), LuceError> {
        let uuid =
            Uuid::from_str(task_id).map_err(|_| LuceError::InvalidTaskId(task_id.to_string()))?;
        let task_id = TaskId::from(uuid);

        let usecase = UpdateTaskStatusUseCase::new(&self.task_repository);
        let input = UpdateTaskStatusInput::new(task_id, status);

        usecase.execute(input).await?;
        Ok(())
    }

    pub async fn assign_task_to_session(
        &self,
        task_id: &str,
        session_id: String,
    ) -> Result<(), LuceError> {
        let uuid =
            Uuid::from_str(task_id).map_err(|_| LuceError::InvalidTaskId(task_id.to_string()))?;
        let task_id = TaskId::from(uuid);

        let usecase = AssignSessionUseCase::new(&self.task_repository);
        let input = AssignSessionInput::assign(task_id, session_id);

        usecase.execute(input).await?;
        Ok(())
    }

    pub async fn unassign_task(&self, task_id: &str) -> Result<(), LuceError> {
        let uuid =
            Uuid::from_str(task_id).map_err(|_| LuceError::InvalidTaskId(task_id.to_string()))?;
        let task_id = TaskId::from(uuid);

        let usecase = AssignSessionUseCase::new(&self.task_repository);
        let input = AssignSessionInput::unassign(task_id);

        usecase.execute(input).await?;
        Ok(())
    }
}
