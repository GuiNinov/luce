use luce_shared::{TaskGraph, TaskId, LuceError};
use crate::repositories::GraphRepository;

pub struct AddDependencyUseCase<R: GraphRepository> {
    repository: R,
}

pub struct AddDependencyInput<'a> {
    pub graph_id: &'a str,
    pub task_id: TaskId,
    pub dependency_id: TaskId,
}

impl<R: GraphRepository> AddDependencyUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: AddDependencyInput<'_>) -> Result<TaskGraph, LuceError> {
        let mut graph = self.repository.load_graph(input.graph_id).await?;
        graph.add_dependency(input.task_id, input.dependency_id)?;
        self.repository.save_graph(&graph, input.graph_id).await?;
        Ok(graph)
    }
}