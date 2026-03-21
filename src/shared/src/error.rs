use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum LuceError {
    #[error("Task not found: {id}")]
    TaskNotFound { id: Uuid },

    #[error("Circular dependency detected")]
    CircularDependency,

    #[error("Invalid task state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Dependency error: {message}")]
    DependencyError { message: String },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid task ID: {0}")]
    InvalidTaskId(String),

    #[error("Database error: {message}")]
    DatabaseError { message: String },
}
