use luce_shared::{TaskGraph, LuceError};
use crate::repositories::GraphRepository;

pub struct CreateGraphUseCase<R: GraphRepository> {
    repository: R,
}

pub struct CreateGraphInput {
    pub id: String,
}

impl<R: GraphRepository> CreateGraphUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: CreateGraphInput) -> Result<TaskGraph, LuceError> {
        let graph = TaskGraph::new();
        self.repository.save_graph(&graph, &input.id).await?;
        Ok(graph)
    }
}