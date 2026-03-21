use luce_shared::{TaskGraph, LuceError};
use crate::repositories::GraphRepository;

pub struct LoadGraphUseCase<R: GraphRepository> {
    repository: R,
}

pub struct LoadGraphInput<'a> {
    pub id: &'a str,
}

impl<R: GraphRepository> LoadGraphUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: LoadGraphInput<'_>) -> Result<TaskGraph, LuceError> {
        self.repository.load_graph(input.id).await
    }
}