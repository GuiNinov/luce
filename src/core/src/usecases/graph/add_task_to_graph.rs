use luce_shared::{TaskGraph, Task, LuceError};
use crate::repositories::GraphRepository;

pub struct AddTaskToGraphUseCase<R: GraphRepository> {
    repository: R,
}

pub struct AddTaskToGraphInput<'a> {
    pub graph_id: &'a str,
    pub task: Task,
}

impl<R: GraphRepository> AddTaskToGraphUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: AddTaskToGraphInput<'_>) -> Result<TaskGraph, LuceError> {
        let mut graph = self.repository.load_graph(input.graph_id).await?;
        graph.add_task(input.task);
        self.repository.save_graph(&graph, input.graph_id).await?;
        Ok(graph)
    }
}