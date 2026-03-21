use luce_shared::{TaskId, LuceError};
use crate::repositories::GraphRepository;

pub struct DetectCyclesUseCase<R: GraphRepository> {
    repository: R,
}

pub struct DetectCyclesInput<'a> {
    pub graph_id: &'a str,
}

impl<R: GraphRepository> DetectCyclesUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: DetectCyclesInput<'_>) -> Result<Option<Vec<TaskId>>, LuceError> {
        let graph = self.repository.load_graph(input.graph_id).await?;
        Ok(graph.find_cycle())
    }
}