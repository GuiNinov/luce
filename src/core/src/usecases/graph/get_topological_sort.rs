use luce_shared::{TaskId, LuceError};
use crate::repositories::GraphRepository;

pub struct GetTopologicalSortUseCase<R: GraphRepository> {
    repository: R,
}

pub struct GetTopologicalSortInput<'a> {
    pub graph_id: &'a str,
}

impl<R: GraphRepository> GetTopologicalSortUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: GetTopologicalSortInput<'_>) -> Result<Vec<TaskId>, LuceError> {
        let graph = self.repository.load_graph(input.graph_id).await?;
        let sorted_tasks = graph.topological_sort()?;
        Ok(sorted_tasks.into_iter().map(|task| task.id).collect())
    }
}