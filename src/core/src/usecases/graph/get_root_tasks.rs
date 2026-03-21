use luce_shared::{TaskId, LuceError};
use crate::repositories::GraphRepository;

pub struct GetRootTasksUseCase<R: GraphRepository> {
    repository: R,
}

pub struct GetRootTasksInput<'a> {
    pub graph_id: &'a str,
}

impl<R: GraphRepository> GetRootTasksUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: GetRootTasksInput<'_>) -> Result<Vec<TaskId>, LuceError> {
        let graph = self.repository.load_graph(input.graph_id).await?;
        Ok(graph.get_root_tasks().into_iter().map(|task| task.id).collect())
    }
}