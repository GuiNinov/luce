-- Initial migration for Luce task management system
-- Creates the core tables for tasks and task graphs

-- Table for individual tasks
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    priority TEXT NOT NULL,
    dependencies TEXT NOT NULL, -- JSON array of task IDs
    dependents TEXT NOT NULL,   -- JSON array of task IDs
    assigned_session TEXT,
    metadata TEXT NOT NULL,     -- JSON object
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT
);

-- Table for task graphs
CREATE TABLE IF NOT EXISTS task_graphs (
    id TEXT PRIMARY KEY,
    graph_data TEXT NOT NULL, -- JSON serialized TaskGraph
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Indexes for better performance
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_assigned_session ON tasks(assigned_session);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at);
CREATE INDEX IF NOT EXISTS idx_task_graphs_created_at ON task_graphs(created_at);