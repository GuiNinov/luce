use clap::{Parser, Subcommand};

mod commands;
mod services;

use commands::*;
use services::LuceService;

#[derive(Parser)]
#[command(name = "luce-cli")]
#[command(about = "A graph-based task management CLI optimized for parallel execution workflows")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Task management commands
    #[command(subcommand)]
    Task(TaskCommands),

    /// Graph operations and visualization
    #[command(subcommand)]
    Graph(GraphCommands),

    /// Session management for multi-user coordination
    #[command(subcommand)]
    Session(SessionCommands),
}

#[derive(Subcommand)]
pub enum TaskCommands {
    /// Create a new task
    Create {
        /// Title of the task
        title: String,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
        /// Task priority (low, normal, high, critical)
        #[arg(short, long, default_value = "normal")]
        priority: String,
        /// Dependencies (comma-separated task IDs)
        #[arg(short = 'D', long)]
        dependencies: Option<String>,
        /// Metadata in key=value format (can be used multiple times)
        #[arg(short, long)]
        metadata: Vec<String>,
    },

    /// List tasks with filtering options
    List {
        /// Filter by status (pending, ready, in-progress, completed, failed, blocked)
        #[arg(short, long)]
        status: Option<String>,
        /// Filter by assigned session
        #[arg(short, long)]
        session: Option<String>,
        /// Filter by priority (low, normal, high, critical)
        #[arg(short, long)]
        priority: Option<String>,
        /// Show only available tasks (ready and unassigned)
        #[arg(short, long)]
        available: bool,
        /// Show only blocked tasks
        #[arg(short, long)]
        blocked: bool,
        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Show detailed information about a specific task
    Show {
        /// Task ID
        task_id: String,
    },

    /// Update an existing task
    Update {
        /// Task ID
        task_id: String,
        /// New title
        #[arg(short, long)]
        title: Option<String>,
        /// New description
        #[arg(short, long)]
        description: Option<String>,
        /// New priority (low, normal, high, critical)
        #[arg(short, long)]
        priority: Option<String>,
        /// New status (pending, ready, in-progress, completed, failed, blocked)
        #[arg(short, long)]
        status: Option<String>,
    },

    /// Start working on a task (set status to in-progress and assign to current session)
    Start {
        /// Task ID
        task_id: String,
        /// Session ID (defaults to current session)
        #[arg(short, long)]
        session: Option<String>,
    },

    /// Complete a task
    Complete {
        /// Task ID
        task_id: String,
    },

    /// Mark a task as failed
    Fail {
        /// Task ID
        task_id: String,
        /// Whether to block dependent tasks
        #[arg(short, long)]
        block_dependents: bool,
    },

    /// Add a dependency between tasks
    AddDependency {
        /// Task ID that depends on another
        task_id: String,
        /// Task ID that is the dependency
        dependency_id: String,
    },

    /// Remove a dependency between tasks
    RemoveDependency {
        /// Task ID that depends on another
        task_id: String,
        /// Task ID that is the dependency
        dependency_id: String,
    },

    /// Assign a task to a session
    Assign {
        /// Task ID
        task_id: String,
        /// Session ID
        session_id: String,
    },

    /// Unassign a task from its session
    Unassign {
        /// Task ID
        task_id: String,
    },

    /// Delete a task
    Delete {
        /// Task ID
        task_id: String,
        /// Force deletion even if task has dependents
        #[arg(short, long)]
        force: bool,
    },

    /// Add metadata to a task
    AddMetadata {
        /// Task ID
        task_id: String,
        /// Key
        key: String,
        /// Value
        value: String,
    },

    /// Remove metadata from a task
    RemoveMetadata {
        /// Task ID
        task_id: String,
        /// Key
        key: String,
    },
}

#[derive(Subcommand)]
pub enum GraphCommands {
    /// Show graph overview and statistics
    Status,

    /// Show visual representation of the task graph
    Show {
        /// Show only tasks with specific status
        #[arg(short, long)]
        status: Option<String>,
        /// Show only tasks assigned to session
        #[arg(short, long)]
        session: Option<String>,
        /// Format output (text, json, dot)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Show dependencies for a specific task
    Dependencies {
        /// Task ID
        task_id: String,
        /// Show recursive dependencies
        #[arg(short, long)]
        recursive: bool,
    },

    /// Show dependents for a specific task
    Dependents {
        /// Task ID
        task_id: String,
        /// Show recursive dependents
        #[arg(short, long)]
        recursive: bool,
    },

    /// Find cycles in the dependency graph
    FindCycles,

    /// Show topological sort of tasks
    TopologicalSort,

    /// Show critical path in the task graph
    CriticalPath,

    /// Clear the entire graph
    Clear {
        /// Confirm deletion
        #[arg(short, long)]
        confirm: bool,
    },

    /// Export graph to different formats
    Export {
        /// Output file path
        #[arg(short, long)]
        output: String,
        /// Export format (json, dot, csv)
        #[arg(short, long, default_value = "json")]
        format: String,
    },

    /// Import graph from file
    Import {
        /// Input file path
        #[arg(short, long)]
        input: String,
        /// Input format (json, csv)
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Merge with existing graph
        #[arg(short, long)]
        merge: bool,
    },
}

#[derive(Subcommand)]
pub enum SessionCommands {
    /// List all active sessions
    List,

    /// Create a new session
    Create {
        /// Session ID
        session_id: String,
        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Show information about a specific session
    Show {
        /// Session ID
        session_id: String,
    },

    /// Set the current session
    Set {
        /// Session ID
        session_id: String,
    },

    /// Get the current session
    Current,

    /// End a session and unassign all its tasks
    End {
        /// Session ID
        session_id: String,
        /// Force end even if tasks are in progress
        #[arg(short, long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let service = LuceService::new().await?;

    match cli.command {
        Commands::Task(task_cmd) => handle_task_command(task_cmd, &service).await,
        Commands::Graph(graph_cmd) => handle_graph_command(graph_cmd, &service).await,
        Commands::Session(session_cmd) => handle_session_command(session_cmd, &service).await,
    }
}
