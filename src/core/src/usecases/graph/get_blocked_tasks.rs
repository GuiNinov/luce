use crate::repositories::GraphRepository;
use luce_shared::{LuceError, TaskId};

pub struct GetBlockedTasksUseCase<R: GraphRepository> {
    repository: R,
}

pub struct GetBlockedTasksInput<'a> {
    pub graph_id: &'a str,
}

impl<R: GraphRepository> GetBlockedTasksUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: GetBlockedTasksInput<'_>) -> Result<Vec<TaskId>, LuceError> {
        let graph = self.repository.load_graph(input.graph_id).await?;
        Ok(graph
            .get_blocked_tasks()
            .into_iter()
            .map(|task| task.id)
            .collect())
    }
}
