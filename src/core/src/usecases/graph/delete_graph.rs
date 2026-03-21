use crate::repositories::GraphRepository;
use luce_shared::LuceError;

pub struct DeleteGraphUseCase<R: GraphRepository> {
    repository: R,
}

pub struct DeleteGraphInput<'a> {
    pub id: &'a str,
}

impl<R: GraphRepository> DeleteGraphUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: DeleteGraphInput<'_>) -> Result<(), LuceError> {
        self.repository.delete_graph(input.id).await
    }
}
