use luce_shared::LuceError;
use crate::repositories::GraphRepository;

pub struct GraphExistsUseCase<R: GraphRepository> {
    repository: R,
}

pub struct GraphExistsInput<'a> {
    pub id: &'a str,
}

impl<R: GraphRepository> GraphExistsUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: GraphExistsInput<'_>) -> Result<bool, LuceError> {
        self.repository.graph_exists(input.id).await
    }
}