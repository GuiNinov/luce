use luce_shared::{TaskGraph, TaskId, LuceError};
use crate::repositories::GraphRepository;

pub struct FailTaskUseCase<R: GraphRepository> {
    repository: R,
}

pub struct FailTaskInput<'a> {
    pub graph_id: &'a str,
    pub task_id: TaskId,
    pub propagate_failure: bool,
}

impl<R: GraphRepository> FailTaskUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: FailTaskInput<'_>) -> Result<(TaskGraph, Vec<TaskId>), LuceError> {
        let mut graph = self.repository.load_graph(input.graph_id).await?;
        let blocked_tasks = graph.fail_task(input.task_id, input.propagate_failure)?;
        self.repository.save_graph(&graph, input.graph_id).await?;
        Ok((graph, blocked_tasks))
    }
}