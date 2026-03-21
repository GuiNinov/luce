use crate::repositories::GraphRepository;
use luce_shared::{LuceError, TaskGraph};

pub struct SaveGraphUseCase<R: GraphRepository> {
    repository: R,
}

pub struct SaveGraphInput<'a> {
    pub graph: &'a TaskGraph,
    pub id: &'a str,
}

impl<R: GraphRepository> SaveGraphUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: SaveGraphInput<'_>) -> Result<(), LuceError> {
        self.repository.save_graph(input.graph, input.id).await
    }
}
