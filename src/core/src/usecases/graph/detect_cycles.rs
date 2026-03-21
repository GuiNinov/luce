use crate::repositories::GraphRepository;
use luce_shared::{LuceError, TaskId};

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

    pub async fn execute(
        &self,
        input: DetectCyclesInput<'_>,
    ) -> Result<Option<Vec<TaskId>>, LuceError> {
        let graph = self.repository.load_graph(input.graph_id).await?;
        let cycles = graph.find_cycles();
        Ok(cycles.first().cloned())
    }
}
