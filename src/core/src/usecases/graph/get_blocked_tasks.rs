use std::collections::HashMap;
use luce_shared::{TaskId, LuceError};
use crate::repositories::GraphRepository;

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

    pub async fn execute(&self, input: GetBlockedTasksInput<'_>) -> Result<HashMap<TaskId, Vec<TaskId>>, LuceError> {
        let graph = self.repository.load_graph(input.graph_id).await?;
        Ok(graph.get_blocked_tasks())
    }
}