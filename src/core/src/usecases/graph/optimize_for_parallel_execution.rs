use crate::repositories::GraphRepository;
use luce_shared::{LuceError, TaskGraph, TaskId, TaskStatus};
use std::collections::HashSet;

pub struct OptimizeForParallelExecutionUseCase<R: GraphRepository> {
    repository: R,
}

pub struct OptimizeForParallelExecutionInput<'a> {
    pub graph_id: &'a str,
}

#[derive(Debug, Clone)]
pub struct ParallelExecutionPlan {
    pub execution_waves: Vec<Vec<TaskId>>,
    pub total_waves: usize,
    pub remaining_tasks: Vec<TaskId>,
    pub estimated_completion_time: u32,
}

impl<R: GraphRepository> OptimizeForParallelExecutionUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        input: OptimizeForParallelExecutionInput<'_>,
    ) -> Result<ParallelExecutionPlan, LuceError> {
        let graph = self.repository.load_graph(input.graph_id).await?;
        let _ready_tasks: Vec<TaskId> = graph
            .get_ready_tasks()
            .into_iter()
            .map(|task| task.id)
            .collect();

        // Create execution waves - groups of tasks that can run simultaneously
        let mut waves = Vec::new();
        let mut remaining_tasks: HashSet<TaskId> = graph.tasks.keys().cloned().collect();
        let mut completed_tasks = HashSet::new();

        // Add already completed/failed tasks to completed set
        for task in graph.tasks.values() {
            if task.status == TaskStatus::Completed || task.status == TaskStatus::Failed {
                completed_tasks.insert(task.id);
                remaining_tasks.remove(&task.id);
            }
        }

        while !remaining_tasks.is_empty() {
            let mut current_wave = Vec::new();
            let mut wave_tasks = HashSet::new();

            for &task_id in &remaining_tasks {
                if let Some(task) = graph.tasks.get(&task_id) {
                    // Check if all dependencies are completed
                    let dependencies_met = task
                        .dependencies
                        .iter()
                        .all(|dep| completed_tasks.contains(dep));

                    if dependencies_met {
                        current_wave.push(task_id);
                        wave_tasks.insert(task_id);
                    }
                }
            }

            if current_wave.is_empty() {
                // No progress possible - might be a cycle or blocked state
                break;
            }

            // Remove wave tasks from remaining and add to completed
            for task_id in &wave_tasks {
                remaining_tasks.remove(task_id);
                completed_tasks.insert(*task_id);
            }

            waves.push(current_wave);
        }

        Ok(ParallelExecutionPlan {
            execution_waves: waves.clone(),
            total_waves: waves.len(),
            remaining_tasks: remaining_tasks.into_iter().collect(),
            estimated_completion_time: self.estimate_completion_time(&graph, &waves),
        })
    }

    fn estimate_completion_time(&self, _graph: &TaskGraph, waves: &[Vec<TaskId>]) -> u32 {
        // Simple estimation: assume each wave takes 1 time unit
        // In a real implementation, this would consider task complexity,
        // historical data, etc.
        waves.len() as u32
    }
}
