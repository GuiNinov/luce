use luce_shared::LuceError;
use crate::repositories::GraphRepository;

pub struct ListGraphsUseCase<R: GraphRepository> {
    repository: R,
}

impl<R: GraphRepository> ListGraphsUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<Vec<String>, LuceError> {
        self.repository.list_graphs().await
    }
}