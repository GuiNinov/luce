use std::collections::HashSet;
use luce_shared::{TaskGraph, TaskId, TaskStatus, LuceError};
use crate::repositories::GraphRepository;

pub struct CalculateParallelOpportunitiesUseCase<R: GraphRepository> {
    repository: R,
}

pub struct CalculateParallelOpportunitiesInput<'a> {
    pub graph_id: &'a str,
}

#[derive(Debug, Clone)]
pub struct ParallelOpportunities {
    pub ready_tasks: Vec<TaskId>,
    pub available_tasks: Vec<TaskId>,
    pub max_parallel_tasks: usize,
    pub conflict_groups: Vec<Vec<TaskId>>,
    pub efficiency_score: f64,
}

impl<R: GraphRepository> CalculateParallelOpportunitiesUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, input: CalculateParallelOpportunitiesInput<'_>) -> Result<ParallelOpportunities, LuceError> {
        let graph = self.repository.load_graph(input.graph_id).await?;
        let ready_tasks: Vec<TaskId> = graph.get_ready_tasks().into_iter().map(|task| task.id).collect();
        let available_tasks: Vec<TaskId> = graph.get_available_tasks().into_iter().map(|task| task.id).collect();
        
        // Calculate how many tasks can be worked on simultaneously
        let max_parallel = ready_tasks.len();
        
        // Group by potential conflicts (tasks that might affect each other)
        let conflict_groups = self.analyze_task_conflicts(&graph, &available_tasks)?;
        
        Ok(ParallelOpportunities {
            ready_tasks,
            available_tasks,
            max_parallel_tasks: max_parallel,
            conflict_groups,
            efficiency_score: self.calculate_efficiency_score(&graph),
        })
    }

    // Helper methods
    fn analyze_task_conflicts(&self, graph: &TaskGraph, tasks: &[TaskId]) -> Result<Vec<Vec<TaskId>>, LuceError> {
        // For now, assume no conflicts - in a real implementation this would
        // analyze file dependencies, resource usage, etc.
        let mut groups = Vec::new();
        
        // Simple grouping: each task in its own group (fully parallel)
        for &task_id in tasks {
            if graph.tasks.contains_key(&task_id) {
                groups.push(vec![task_id]);
            }
        }
        
        Ok(groups)
    }

    fn calculate_efficiency_score(&self, graph: &TaskGraph) -> f64 {
        let total_tasks = graph.tasks.len();
        if total_tasks == 0 {
            return 1.0;
        }
        
        let ready_tasks = graph.get_ready_tasks().len();
        let completed_tasks = graph.tasks.values()
            .filter(|t| matches!(t.status, TaskStatus::Completed))
            .count();
            
        let ready_ratio = ready_tasks as f64 / total_tasks as f64;
        let progress_ratio = completed_tasks as f64 / total_tasks as f64;
        
        // Efficiency is a combination of how much is ready to work on
        // and how much progress has been made
        (ready_ratio * 0.6) + (progress_ratio * 0.4)
    }
}