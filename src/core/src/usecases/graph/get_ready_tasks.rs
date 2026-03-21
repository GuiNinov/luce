use crate::repositories::GraphRepository;
use luce_shared::{LuceError, TaskId};

pub struct GetReadyTasksUseCase<R: GraphRepository> {
    repository: R,
}

pub struct GetReadyTasksInput<'a> {
    pub graph_id: &'a str,
}

impl<R: GraphRepository> GetReadyTasksUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: GetReadyTasksInput<'_>) -> Result<Vec<TaskId>, LuceError> {
        let graph = self.repository.load_graph(input.graph_id).await?;
        Ok(graph
            .get_ready_tasks()
            .into_iter()
            .map(|task| task.id)
            .collect())
    }
}
