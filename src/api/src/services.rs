use std::sync::Arc;

use luce_core::{
    CreateTaskInput, CreateTaskUseCase, GetTaskInput, GetTaskUseCase, ListTasksInput,
    ListTasksUseCase, SqliteTaskRepository, TaskFilter, TaskRepository, UpdateTaskStatusInput,
    UpdateTaskStatusUseCase, UseCase, CredentialService,
};
use luce_shared::LuceError;
use luce_shared::{Task, TaskId, TaskStatus};
use sqlx::SqlitePool;

use crate::ApiResult;

pub struct TaskService {
    task_repo: Arc<SqliteTaskRepository>,
    create_task_uc: CreateTaskUseCase<SqliteTaskRepository>,
    get_task_uc: GetTaskUseCase<SqliteTaskRepository>,
    list_tasks_uc: ListTasksUseCase<SqliteTaskRepository>,
    update_status_uc: UpdateTaskStatusUseCase<SqliteTaskRepository>,
    pub credential_service: Arc<CredentialService>,
}

impl TaskService {
    pub async fn new(pool: SqlitePool) -> Result<Self, LuceError> {
        let task_repo1 = SqliteTaskRepository::from_pool(pool.clone());
        let task_repo2 = SqliteTaskRepository::from_pool(pool.clone());
        let task_repo3 = SqliteTaskRepository::from_pool(pool.clone());
        let task_repo4 = SqliteTaskRepository::from_pool(pool.clone());
        let task_repo5 = SqliteTaskRepository::from_pool(pool.clone());

        let credential_service = Arc::new(CredentialService::from_pool(pool));

        Ok(Self {
            create_task_uc: CreateTaskUseCase::new(task_repo1),
            get_task_uc: GetTaskUseCase::new(task_repo2),
            list_tasks_uc: ListTasksUseCase::new(task_repo3),
            update_status_uc: UpdateTaskStatusUseCase::new(task_repo4),
            task_repo: Arc::new(task_repo5),
            credential_service,
        })
    }

    pub async fn create_task(
        &self,
        title: String,
        description: Option<String>,
        _dependencies: Vec<TaskId>, // TODO: Handle dependencies
    ) -> ApiResult<Task> {
        let mut input = CreateTaskInput::new(title);
        if let Some(desc) = description {
            input = input.with_description(desc);
        }

        self.create_task_uc.execute(input).await
    }

    pub async fn get_task(&self, task_id: TaskId) -> ApiResult<Task> {
        let input = GetTaskInput { task_id };
        self.get_task_uc.execute(input).await
    }

    pub async fn update_task(
        &self,
        task_id: TaskId,
        _title: Option<String>,
        _description: Option<String>,
        status: Option<TaskStatus>,
    ) -> ApiResult<Task> {
        // For now, only support status updates
        if let Some(status) = status {
            let input = UpdateTaskStatusInput {
                task_id,
                new_status: status,
            };
            self.update_status_uc.execute(input).await
        } else {
            // If no status update, just return the current task
            let input = GetTaskInput { task_id };
            self.get_task_uc.execute(input).await
        }
    }

    pub async fn delete_task(&self, task_id: TaskId) -> ApiResult<()> {
        // TODO: Implement delete functionality when available
        self.task_repo.delete_task(task_id).await
    }

    pub async fn list_tasks(
        &self,
        status_filter: Option<TaskStatus>,
        _limit: Option<usize>,
        _offset: Option<usize>,
    ) -> ApiResult<(Vec<Task>, usize)> {
        let filter = status_filter.map_or(TaskFilter::All, |status| TaskFilter::ByStatus(status));
        let input = ListTasksInput { filter };
        let tasks = self.list_tasks_uc.execute(input).await?;
        let total_count = tasks.len();
        Ok((tasks, total_count))
    }

    pub async fn mark_task_completed(&self, task_id: TaskId) -> ApiResult<Task> {
        let input = UpdateTaskStatusInput {
            task_id,
            new_status: TaskStatus::Completed,
        };
        self.update_status_uc.execute(input).await
    }

    pub async fn get_ready_tasks(&self) -> ApiResult<Vec<Task>> {
        // For now, return tasks with Pending status
        let filter = TaskFilter::ByStatus(TaskStatus::Pending);
        let input = ListTasksInput { filter };
        let tasks = self.list_tasks_uc.execute(input).await?;
        Ok(tasks)
    }
}
