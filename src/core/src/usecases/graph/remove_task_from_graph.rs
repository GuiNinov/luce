use luce_shared::{TaskGraph, TaskId, LuceError};
use crate::repositories::GraphRepository;

pub struct RemoveTaskFromGraphUseCase<R: GraphRepository> {
    repository: R,
}

pub struct RemoveTaskFromGraphInput<'a> {
    pub graph_id: &'a str,
    pub task_id: TaskId,
}

impl<R: GraphRepository> RemoveTaskFromGraphUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: RemoveTaskFromGraphInput<'_>) -> Result<TaskGraph, LuceError> {
        let mut graph = self.repository.load_graph(input.graph_id).await?;
        graph.remove_task(&input.task_id)
            .ok_or_else(|| LuceError::TaskNotFound { id: input.task_id })?;
        self.repository.save_graph(&graph, input.graph_id).await?;
        Ok(graph)
    }
}