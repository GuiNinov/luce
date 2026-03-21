use crate::repositories::GraphRepository;
use luce_shared::{LuceError, TaskId};

pub struct GetAvailableTasksUseCase<R: GraphRepository> {
    repository: R,
}

pub struct GetAvailableTasksInput<'a> {
    pub graph_id: &'a str,
    pub session_id: Option<&'a str>,
}

impl<R: GraphRepository> GetAvailableTasksUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        input: GetAvailableTasksInput<'_>,
    ) -> Result<Vec<TaskId>, LuceError> {
        let graph = self.repository.load_graph(input.graph_id).await?;
        Ok(graph
            .get_available_tasks()
            .into_iter()
            .map(|task| task.id)
            .collect())
    }
}
