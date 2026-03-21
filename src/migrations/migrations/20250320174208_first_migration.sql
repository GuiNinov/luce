-- Initial migration for Luce task management system
-- Creates the core tables for simplified task-only system with dependency support

-- Table for individual tasks
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    priority TEXT NOT NULL,
    assigned_session TEXT,
    metadata TEXT NOT NULL,     -- JSON object
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT
);

-- Table for task dependencies (many-to-many relationship)
CREATE TABLE IF NOT EXISTS task_dependencies (
    id TEXT PRIMARY KEY,             -- UUID as text for consistency
    task_id TEXT NOT NULL,           -- The task that has dependencies
    dependency_id TEXT NOT NULL,     -- The task that must be completed first
    created_at TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (dependency_id) REFERENCES tasks(id) ON DELETE CASCADE,
    UNIQUE(task_id, dependency_id)   -- Prevent duplicate dependencies
);

-- Indexes for better performance
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_assigned_session ON tasks(assigned_session);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at);

-- Indexes for task dependencies
CREATE INDEX IF NOT EXISTS idx_task_dependencies_task_id ON task_dependencies(task_id);
CREATE INDEX IF NOT EXISTS idx_task_dependencies_dependency_id ON task_dependencies(dependency_id);

-- Table for integration credentials (encrypted storage)
CREATE TABLE IF NOT EXISTS integration_credentials (
    id TEXT PRIMARY KEY,                    -- UUID
    integration_type TEXT NOT NULL,         -- 'github', 'slack', 'linear', etc.
    name TEXT NOT NULL,                     -- User-friendly name (e.g., 'Work GitHub', 'Personal GitHub')
    encrypted_data TEXT NOT NULL,           -- Encrypted JSON with credentials
    is_active BOOLEAN NOT NULL DEFAULT 1,  -- Whether this credential is active
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_used_at TEXT,                      -- When this credential was last used
    UNIQUE(integration_type, name)          -- Prevent duplicate names per integration type
);

-- Table for integration configurations and status
CREATE TABLE IF NOT EXISTS integrations (
    id TEXT PRIMARY KEY,                    -- UUID
    integration_type TEXT NOT NULL,         -- 'github', 'slack', 'linear', etc.
    credential_id TEXT,                     -- FK to integration_credentials
    config_data TEXT NOT NULL,              -- JSON with integration-specific config
    is_enabled BOOLEAN NOT NULL DEFAULT 0, -- Whether integration is enabled
    last_sync_at TEXT,                      -- When last sync occurred
    sync_status TEXT DEFAULT 'never',      -- 'never', 'success', 'error', 'in_progress'
    sync_error TEXT,                        -- Last sync error if any
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (credential_id) REFERENCES integration_credentials(id) ON DELETE SET NULL,
    UNIQUE(integration_type)                -- One integration config per type
);

-- Table for task attachments to external resources
CREATE TABLE IF NOT EXISTS task_attachments (
    id TEXT PRIMARY KEY,                    -- UUID
    task_id TEXT NOT NULL,                  -- FK to tasks
    integration_type TEXT NOT NULL,         -- 'github', 'slack', 'linear', etc.
    external_id TEXT NOT NULL,              -- External resource ID (issue #, PR #, etc.)
    external_url TEXT,                      -- Direct URL to external resource
    attachment_data TEXT NOT NULL,          -- JSON with attachment-specific data
    attachment_type TEXT NOT NULL,          -- 'issue', 'pr', 'thread', 'ticket', etc.
    status TEXT DEFAULT 'active',           -- 'active', 'deleted', 'archived'
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    synced_at TEXT,                         -- When attachment data was last synced
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    UNIQUE(task_id, integration_type, external_id) -- Prevent duplicate attachments
);

-- Indexes for integration credentials
CREATE INDEX IF NOT EXISTS idx_integration_credentials_type ON integration_credentials(integration_type);
CREATE INDEX IF NOT EXISTS idx_integration_credentials_active ON integration_credentials(is_active);

-- Indexes for integrations  
CREATE INDEX IF NOT EXISTS idx_integrations_type ON integrations(integration_type);
CREATE INDEX IF NOT EXISTS idx_integrations_enabled ON integrations(is_enabled);
CREATE INDEX IF NOT EXISTS idx_integrations_credential_id ON integrations(credential_id);

-- Indexes for task attachments
CREATE INDEX IF NOT EXISTS idx_task_attachments_task_id ON task_attachments(task_id);
CREATE INDEX IF NOT EXISTS idx_task_attachments_integration_type ON task_attachments(integration_type);
CREATE INDEX IF NOT EXISTS idx_task_attachments_external_id ON task_attachments(integration_type, external_id);
CREATE INDEX IF NOT EXISTS idx_task_attachments_status ON task_attachments(status);