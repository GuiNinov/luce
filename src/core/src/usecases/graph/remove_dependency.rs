use crate::repositories::GraphRepository;
use luce_shared::{LuceError, TaskGraph, TaskId};

pub struct RemoveDependencyUseCase<R: GraphRepository> {
    repository: R,
}

pub struct RemoveDependencyInput<'a> {
    pub graph_id: &'a str,
    pub task_id: TaskId,
    pub dependency_id: TaskId,
}

impl<R: GraphRepository> RemoveDependencyUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: RemoveDependencyInput<'_>) -> Result<TaskGraph, LuceError> {
        let mut graph = self.repository.load_graph(input.graph_id).await?;
        graph.remove_dependency(input.task_id, input.dependency_id)?;
        self.repository.save_graph(&graph, input.graph_id).await?;
        Ok(graph)
    }
}
