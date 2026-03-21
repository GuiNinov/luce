use luce_shared::{TaskId, LuceError};
use crate::repositories::GraphRepository;

pub struct GetLeafTasksUseCase<R: GraphRepository> {
    repository: R,
}

pub struct GetLeafTasksInput<'a> {
    pub graph_id: &'a str,
}

impl<R: GraphRepository> GetLeafTasksUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: GetLeafTasksInput<'_>) -> Result<Vec<TaskId>, LuceError> {
        let graph = self.repository.load_graph(input.graph_id).await?;
        Ok(graph.get_leaf_tasks())
    }
}