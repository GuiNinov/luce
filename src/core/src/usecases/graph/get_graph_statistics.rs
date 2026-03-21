use luce_shared::{TaskStatus, LuceError};
use crate::repositories::GraphRepository;

pub struct GetGraphStatisticsUseCase<R: GraphRepository> {
    repository: R,
}

pub struct GetGraphStatisticsInput<'a> {
    pub graph_id: &'a str,
}

#[derive(Debug, Clone)]
pub struct GraphStatistics {
    pub total_tasks: usize,
    pub pending_tasks: usize,
    pub ready_tasks: usize,
    pub in_progress_tasks: usize,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub blocked_tasks: usize,
    pub progress_percentage: f64,
}

impl<R: GraphRepository> GetGraphStatisticsUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: GetGraphStatisticsInput<'_>) -> Result<GraphStatistics, LuceError> {
        let graph = self.repository.load_graph(input.graph_id).await?;
        
        let total_tasks = graph.tasks.len();
        let pending_tasks = graph.tasks.values().filter(|t| matches!(t.status, TaskStatus::Pending)).count();
        let ready_tasks = graph.get_ready_tasks().len();
        let in_progress_tasks = graph.tasks.values().filter(|t| matches!(t.status, TaskStatus::InProgress)).count();
        let completed_tasks = graph.tasks.values().filter(|t| matches!(t.status, TaskStatus::Completed)).count();
        let failed_tasks = graph.tasks.values().filter(|t| matches!(t.status, TaskStatus::Failed)).count();
        let blocked_tasks = graph.tasks.values().filter(|t| matches!(t.status, TaskStatus::Blocked)).count();
        
        let progress_percentage = if total_tasks > 0 {
            (completed_tasks as f64 / total_tasks as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(GraphStatistics {
            total_tasks,
            pending_tasks,
            ready_tasks,
            in_progress_tasks,
            completed_tasks,
            failed_tasks,
            blocked_tasks,
            progress_percentage,
        })
    }
}