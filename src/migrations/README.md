# Luce Migrations

Database migration management for the Luce task management system.

## Overview

This package provides a simple, timestamp-based migration system for managing database schema changes in Luce. It supports:

- **Ordered Migrations**: Migrations are applied in timestamp order
- **Migration Tracking**: Keeps track of applied migrations in the database
- **Rollback Support**: Can rollback the last applied migration
- **SQLite Integration**: Built specifically for SQLite databases
- **CLI Tools**: Command-line tools for migration management

## Migration File Format

Migration files follow the naming convention:
```
YYYYMMDDHHMMSS_description.sql
```

Examples:
- `20250320174208_first_migration.sql`
- `20250320174215_add_task_metrics.sql`
- `20250320174230_add_user_preferences.sql`

## Usage

### 1. Generate a New Migration

```bash
cargo run --bin generate "create users table"
```

This creates a new migration file with a timestamp and sanitized description.

### 2. Apply Migrations

```bash
cargo run --bin migrate up sqlite:./luce.db
```

Applies all pending migrations to the database.

### 3. Check Migration Status

```bash
cargo run --bin migrate status sqlite:./luce.db
```

Shows which migrations have been applied and which are pending.

### 4. Rollback Last Migration

```bash
cargo run --bin migrate rollback sqlite:./luce.db
```

Rolls back the most recently applied migration (removes from tracking table).

## API Usage

```rust
use luce_migrations::{MigrationApplier, MigrationRunner};
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let applier = MigrationApplier::new("sqlite:./luce.db").await?;
    
    // Apply all pending migrations
    let count = applier.run_migrations(Path::new("migrations")).await?;
    println!("Applied {} migrations", count);
    
    // Check status
    let applied = applier.get_applied_migrations().await?;
    let pending = applier.get_pending_migrations(Path::new("migrations")).await?;
    
    println!("Applied: {}, Pending: {}", applied.len(), pending.len());
    
    Ok(())
}
```

## Migration Structure

### Core Components

- **`Migration`** - Represents a single migration with metadata
- **`MigrationApplier`** - Handles applying and tracking migrations
- **`MigrationGenerator`** - Generates new migration files
- **`MigrationRunner`** - Trait for migration operations

### Migration Tracking

Migrations are tracked in the `__luce_migrations` table:
```sql
CREATE TABLE __luce_migrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    applied_at TEXT NOT NULL
);
```

### Built-in Migrations

The package includes several built-in migrations:

1. **`20250320174208_first_migration.sql`**
   - Creates core `tasks` and `task_graphs` tables
   - Adds performance indexes

2. **`20250320174215_add_task_metrics.sql`**
   - Adds metrics tracking tables
   - Supports performance monitoring

3. **`20250320174230_add_user_preferences.sql`**
   - Adds user preferences and workspace configuration
   - Supports multi-user environments

## CLI Commands

### Migration Tool (`migrate`)
```bash
migrate <COMMAND> <DATABASE_URL> [MIGRATIONS_DIR]

Commands:
  up, migrate    Apply all pending migrations
  status         Show migration status  
  rollback       Rollback the last applied migration
```

### Generator Tool (`generate`)
```bash
generate <DESCRIPTION> [MIGRATIONS_DIR]

Examples:
  generate "create users table"
  generate "add indexes to tasks" ./my_migrations
```

## Testing

The package includes comprehensive tests for all components:

```bash
cargo test -p luce-migrations
```

Tests cover:
- Migration file parsing and validation
- Database operations and tracking
- Error handling and edge cases
- File system operations
- CLI tool functionality