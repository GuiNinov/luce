use async_trait::async_trait;
use chrono::Utc;
use sqlx::{Row, SqlitePool};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::migration::{Migration, MigrationError};

#[async_trait]
pub trait MigrationRunner {
    async fn run_migrations(&self, migrations_dir: &Path) -> Result<usize, MigrationError>;
    async fn get_applied_migrations(&self) -> Result<Vec<Migration>, MigrationError>;
    async fn get_pending_migrations(&self, migrations_dir: &Path) -> Result<Vec<Migration>, MigrationError>;
    async fn rollback_last_migration(&self) -> Result<Option<Migration>, MigrationError>;
}

pub struct MigrationApplier {
    pool: SqlitePool,
}

impl MigrationApplier {
    pub async fn new(database_url: &str) -> Result<Self, MigrationError> {
        let pool = SqlitePool::connect(database_url)
            .await
            .map_err(|e| MigrationError::DatabaseError {
                error: e.to_string(),
            })?;

        let applier = Self { pool };
        applier.create_migrations_table().await?;
        Ok(applier)
    }

    async fn create_migrations_table(&self) -> Result<(), MigrationError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS __luce_migrations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                applied_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| MigrationError::DatabaseError {
            error: e.to_string(),
        })?;

        Ok(())
    }

    async fn load_migrations_from_dir(&self, migrations_dir: &Path) -> Result<Vec<Migration>, MigrationError> {
        let mut migrations = Vec::new();

        let read_dir = std::fs::read_dir(migrations_dir)
            .map_err(|e| MigrationError::ReadError {
                error: format!("Failed to read directory {}: {}", migrations_dir.display(), e),
            })?;

        for entry in read_dir {
            let entry = entry.map_err(|e| MigrationError::ReadError {
                error: e.to_string(),
            })?;

            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("sql") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| MigrationError::ReadError {
                        error: format!("Failed to read {}: {}", path.display(), e),
                    })?;

                let migration = Migration::new(path, content)?;
                migrations.push(migration);
            }
        }

        migrations.sort();
        Ok(migrations)
    }

    async fn get_applied_migration_names(&self) -> Result<HashSet<String>, MigrationError> {
        let rows = sqlx::query("SELECT name FROM __luce_migrations ORDER BY applied_at")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MigrationError::DatabaseError {
                error: e.to_string(),
            })?;

        let names = rows
            .into_iter()
            .map(|row| row.get::<String, _>("name"))
            .collect();

        Ok(names)
    }

    async fn apply_migration(&self, migration: &Migration) -> Result<(), MigrationError> {
        let mut transaction = self.pool.begin()
            .await
            .map_err(|e| MigrationError::DatabaseError {
                error: e.to_string(),
            })?;

        // Execute the migration SQL
        sqlx::query(&migration.content)
            .execute(&mut *transaction)
            .await
            .map_err(|e| MigrationError::DatabaseError {
                error: format!("Failed to execute migration {}: {}", migration.name, e),
            })?;

        // Record the migration as applied
        let now = Utc::now().to_rfc3339();
        sqlx::query("INSERT INTO __luce_migrations (name, applied_at) VALUES (?, ?)")
            .bind(&migration.name)
            .bind(&now)
            .execute(&mut *transaction)
            .await
            .map_err(|e| MigrationError::DatabaseError {
                error: format!("Failed to record migration {}: {}", migration.name, e),
            })?;

        transaction.commit()
            .await
            .map_err(|e| MigrationError::DatabaseError {
                error: e.to_string(),
            })?;

        Ok(())
    }

    async fn remove_migration_record(&self, migration_name: &str) -> Result<(), MigrationError> {
        sqlx::query("DELETE FROM __luce_migrations WHERE name = ?")
            .bind(migration_name)
            .execute(&self.pool)
            .await
            .map_err(|e| MigrationError::DatabaseError {
                error: e.to_string(),
            })?;

        Ok(())
    }
}

#[async_trait]
impl MigrationRunner for MigrationApplier {
    async fn run_migrations(&self, migrations_dir: &Path) -> Result<usize, MigrationError> {
        let migrations = self.load_migrations_from_dir(migrations_dir).await?;
        let applied_names = self.get_applied_migration_names().await?;

        let pending_migrations: Vec<_> = migrations
            .into_iter()
            .filter(|m| !applied_names.contains(&m.name))
            .collect();

        let count = pending_migrations.len();
        
        for migration in pending_migrations {
            println!("Applying migration: {}", migration.name);
            self.apply_migration(&migration).await?;
            println!("✓ Applied: {}", migration.description);
        }

        if count == 0 {
            println!("No pending migrations to apply.");
        } else {
            println!("Applied {} migration(s) successfully.", count);
        }

        Ok(count)
    }

    async fn get_applied_migrations(&self) -> Result<Vec<Migration>, MigrationError> {
        let rows = sqlx::query("SELECT name, applied_at FROM __luce_migrations ORDER BY applied_at")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| MigrationError::DatabaseError {
                error: e.to_string(),
            })?;

        let mut migrations = Vec::new();
        for row in rows {
            let name: String = row.get("name");
            let applied_at_str: String = row.get("applied_at");
            let applied_at = chrono::DateTime::parse_from_rfc3339(&applied_at_str)
                .map_err(|e| MigrationError::DatabaseError {
                    error: format!("Invalid datetime format: {}", e),
                })?
                .with_timezone(&Utc);

            // Create a minimal migration record (we don't have the original file content)
            let migration = Migration {
                name: name.clone(),
                timestamp: name[0..14].to_string(), // Extract timestamp from filename
                description: "Applied migration".to_string(),
                content: String::new(),
                file_path: PathBuf::from(&name),
                applied_at: Some(applied_at),
            };

            migrations.push(migration);
        }

        Ok(migrations)
    }

    async fn get_pending_migrations(&self, migrations_dir: &Path) -> Result<Vec<Migration>, MigrationError> {
        let migrations = self.load_migrations_from_dir(migrations_dir).await?;
        let applied_names = self.get_applied_migration_names().await?;

        let pending_migrations: Vec<_> = migrations
            .into_iter()
            .filter(|m| !applied_names.contains(&m.name))
            .collect();

        Ok(pending_migrations)
    }

    async fn rollback_last_migration(&self) -> Result<Option<Migration>, MigrationError> {
        // Get the last applied migration
        let last_migration_row = sqlx::query("SELECT name FROM __luce_migrations ORDER BY applied_at DESC LIMIT 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| MigrationError::DatabaseError {
                error: e.to_string(),
            })?;

        if let Some(row) = last_migration_row {
            let migration_name: String = row.get("name");
            
            // Note: This is a simple implementation that just removes the record.
            // In a full implementation, you'd want to support rollback scripts.
            println!("Rolling back migration: {}", migration_name);
            println!("Warning: This only removes the migration record. Manual rollback may be required.");
            
            self.remove_migration_record(&migration_name).await?;
            
            let migration = Migration {
                name: migration_name.clone(),
                timestamp: migration_name[0..14].to_string(),
                description: "Rolled back migration".to_string(),
                content: String::new(),
                file_path: PathBuf::from(&migration_name),
                applied_at: None,
            };
            
            Ok(Some(migration))
        } else {
            println!("No migrations to rollback.");
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, NamedTempFile};
    use std::fs;

    async fn create_test_applier() -> MigrationApplier {
        // Use in-memory database for tests
        let db_url = "sqlite::memory:";
        MigrationApplier::new(db_url).await.unwrap()
    }

    #[tokio::test]
    async fn test_create_migrations_table() {
        let applier = create_test_applier().await;
        
        // Verify the migrations table exists
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE name = '__luce_migrations'")
            .fetch_one(&applier.pool)
            .await
            .unwrap();
        
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_load_migrations_from_dir() {
        let applier = create_test_applier().await;
        let temp_dir = tempdir().unwrap();
        
        // Create test migration files
        fs::write(
            temp_dir.path().join("20250320174208_first_migration.sql"),
            "CREATE TABLE test1 (id INTEGER);"
        ).unwrap();
        
        fs::write(
            temp_dir.path().join("20250320174209_second_migration.sql"),
            "CREATE TABLE test2 (id INTEGER);"
        ).unwrap();
        
        let migrations = applier.load_migrations_from_dir(temp_dir.path()).await.unwrap();
        
        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0].name, "20250320174208_first_migration.sql");
        assert_eq!(migrations[1].name, "20250320174209_second_migration.sql");
        assert!(migrations[0] < migrations[1]); // Test ordering
    }

    #[tokio::test]
    async fn test_run_migrations() {
        let applier = create_test_applier().await;
        let temp_dir = tempdir().unwrap();
        
        // Create test migration
        fs::write(
            temp_dir.path().join("20250320174208_create_test_table.sql"),
            "CREATE TABLE migration_test (id INTEGER PRIMARY KEY, name TEXT);"
        ).unwrap();
        
        let count = applier.run_migrations(temp_dir.path()).await.unwrap();
        assert_eq!(count, 1);
        
        // Verify table was created
        let table_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE name = 'migration_test'")
            .fetch_one(&applier.pool)
            .await
            .unwrap();
        assert_eq!(table_count, 1);
        
        // Verify migration was recorded
        let migration_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM __luce_migrations")
            .fetch_one(&applier.pool)
            .await
            .unwrap();
        assert_eq!(migration_count, 1);
    }

    #[tokio::test]
    async fn test_get_applied_migrations() {
        let applier = create_test_applier().await;
        let temp_dir = tempdir().unwrap();
        
        // Create and apply a migration
        fs::write(
            temp_dir.path().join("20250320174208_test.sql"),
            "CREATE TABLE test (id INTEGER);"
        ).unwrap();
        
        applier.run_migrations(temp_dir.path()).await.unwrap();
        
        let applied = applier.get_applied_migrations().await.unwrap();
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].name, "20250320174208_test.sql");
        assert!(applied[0].is_applied());
    }

    #[tokio::test]
    async fn test_get_pending_migrations() {
        let applier = create_test_applier().await;
        let temp_dir = tempdir().unwrap();
        
        // Create migrations
        fs::write(
            temp_dir.path().join("20250320174208_first.sql"),
            "CREATE TABLE test1 (id INTEGER);"
        ).unwrap();
        
        fs::write(
            temp_dir.path().join("20250320174209_second.sql"),
            "CREATE TABLE test2 (id INTEGER);"
        ).unwrap();
        
        // Apply only the first one
        let migration = Migration::new(
            temp_dir.path().join("20250320174208_first.sql"),
            "CREATE TABLE test1 (id INTEGER);".to_string()
        ).unwrap();
        applier.apply_migration(&migration).await.unwrap();
        
        let pending = applier.get_pending_migrations(temp_dir.path()).await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].name, "20250320174209_second.sql");
    }

    #[tokio::test]
    async fn test_rollback_last_migration() {
        let applier = create_test_applier().await;
        let temp_dir = tempdir().unwrap();
        
        // Create and apply a migration
        fs::write(
            temp_dir.path().join("20250320174208_test.sql"),
            "CREATE TABLE test (id INTEGER);"
        ).unwrap();
        
        applier.run_migrations(temp_dir.path()).await.unwrap();
        
        // Rollback
        let rolled_back = applier.rollback_last_migration().await.unwrap();
        assert!(rolled_back.is_some());
        assert_eq!(rolled_back.unwrap().name, "20250320174208_test.sql");
        
        // Verify migration record was removed
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM __luce_migrations")
            .fetch_one(&applier.pool)
            .await
            .unwrap();
        assert_eq!(count, 0);
    }
}