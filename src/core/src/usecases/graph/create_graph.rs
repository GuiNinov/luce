use crate::repositories::GraphRepository;
use luce_shared::{LuceError, TaskGraph};

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
