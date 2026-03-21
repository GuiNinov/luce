pub mod repositories;
pub mod usecases;

pub use repositories::{TaskRepository, GraphRepository, SqliteTaskRepository, SqliteGraphRepository};
pub use usecases::{UseCase, GraphUseCase};
pub use usecases::task::*;
pub use usecases::graph_usecase::{GraphStatistics, ParallelOpportunities, ParallelExecutionPlan};

// Re-export shared types for convenience
pub use luce_shared::{Task, TaskId, TaskStatus, TaskPriority, TaskGraph, LuceError};