pub mod repositories;
pub mod usecases;

pub use repositories::{
    GraphRepository, SqliteGraphRepository, SqliteTaskRepository, TaskRepository,
};
pub use usecases::graph::{
    AddDependencyInput, AddDependencyUseCase, AddTaskToGraphInput, AddTaskToGraphUseCase,
    CreateGraphInput, CreateGraphUseCase, GetReadyTasksInput, GetReadyTasksUseCase,
    GraphExistsInput, GraphExistsUseCase, GraphStatistics, LoadGraphInput, LoadGraphUseCase,
    ParallelExecutionPlan, ParallelOpportunities, RemoveTaskFromGraphInput,
    RemoveTaskFromGraphUseCase,
};
pub use usecases::task::*;
pub use usecases::UseCase;

// Re-export shared types for convenience
pub use luce_shared::{LuceError, Task, TaskGraph, TaskId, TaskPriority, TaskStatus};
