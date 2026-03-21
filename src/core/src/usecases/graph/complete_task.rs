use luce_shared::{TaskGraph, TaskId, LuceError};
use crate::repositories::GraphRepository;

pub struct CompleteTaskUseCase<R: GraphRepository> {
    repository: R,
}

pub struct CompleteTaskInput<'a> {
    pub graph_id: &'a str,
    pub task_id: TaskId,
}

impl<R: GraphRepository> CompleteTaskUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: CompleteTaskInput<'_>) -> Result<(TaskGraph, Vec<TaskId>), LuceError> {
        let mut graph = self.repository.load_graph(input.graph_id).await?;
        let newly_ready = graph.complete_task(input.task_id)?;
        self.repository.save_graph(&graph, input.graph_id).await?;
        Ok((graph, newly_ready))
    }
}