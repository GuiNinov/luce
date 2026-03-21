use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::io;
use uuid::Uuid;

use super::TaskRepository;
use luce_shared::{LuceError, Task, TaskId, TaskPriority, TaskStatus};

pub struct SqliteTaskRepository {
    pool: SqlitePool,
}

impl SqliteTaskRepository {
    pub async fn new(database_url: &str) -> Result<Self, LuceError> {
        let pool = SqlitePool::connect(database_url)
            .await
            .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))?;

        Ok(Self { pool })
    }

    fn serialize_metadata(metadata: &HashMap<String, String>) -> Result<String, LuceError> {
        serde_json::to_string(metadata).map_err(LuceError::SerializationError)
    }

    fn deserialize_metadata(json: &str) -> Result<HashMap<String, String>, LuceError> {
        serde_json::from_str(json).map_err(LuceError::SerializationError)
    }
}

#[async_trait]
impl TaskRepository for SqliteTaskRepository {
    async fn save_task(&self, task: &Task) -> Result<(), LuceError> {
        let metadata_json = Self::serialize_metadata(&task.metadata)?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO tasks (
                id, title, description, status, priority,
                assigned_session, metadata, created_at, updated_at, started_at, completed_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(task.id.to_string())
        .bind(&task.title)
        .bind(&task.description)
        .bind(
            serde_json::to_string(&task.status)
                .map_err(LuceError::SerializationError)?
                .trim_matches('"'),
        )
        .bind(
            serde_json::to_string(&task.priority)
                .map_err(LuceError::SerializationError)?
                .trim_matches('"'),
        )
        .bind(&task.assigned_session)
        .bind(metadata_json)
        .bind(task.created_at.to_rfc3339())
        .bind(task.updated_at.to_rfc3339())
        .bind(task.started_at.map(|dt| dt.to_rfc3339()))
        .bind(task.completed_at.map(|dt| dt.to_rfc3339()))
        .execute(&self.pool)
        .await
        .map_err(|e| LuceError::IoError(io::Error::new(io::ErrorKind::Other, e.to_string())))?;

        Ok(())
    }

    async fn get_task(&self, id: TaskId) -> Result<Task, LuceError> {
        let row = sqlx::query("SELECT * FROM tasks WHERE id = ?")
            .bind(id.to_string())
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => LuceError::TaskNotFound { id },
                _ => LuceError::IoError(io::Error::other(e.to_string())),
            })?;

        let task_id: TaskId = Uuid::parse_str(row.get("id")).map_err(|e| {
            LuceError::IoError(io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
        })?;

        let status: TaskStatus =
            serde_json::from_str(&format!("\"{}\"", row.get::<String, _>("status"))).map_err(
                |e| {
                    LuceError::IoError(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid status: {}", e),
                    ))
                },
            )?;

        let priority: TaskPriority =
            serde_json::from_str(&format!("\"{}\"", row.get::<String, _>("priority"))).map_err(
                |e| {
                    LuceError::IoError(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid priority: {}", e),
                    ))
                },
            )?;

        let metadata = Self::deserialize_metadata(row.get("metadata"))?;

        let created_at: DateTime<Utc> = DateTime::parse_from_rfc3339(row.get("created_at"))
            .map_err(|e| {
                LuceError::IoError(io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
            })?
            .with_timezone(&Utc);

        let updated_at: DateTime<Utc> = DateTime::parse_from_rfc3339(row.get("updated_at"))
            .map_err(|e| {
                LuceError::IoError(io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
            })?
            .with_timezone(&Utc);

        let started_at = if let Some(started_str) = row.get::<Option<String>, _>("started_at") {
            Some(
                DateTime::parse_from_rfc3339(&started_str)
                    .map_err(|e| {
                        LuceError::IoError(io::Error::new(
                            io::ErrorKind::InvalidData,
                            e.to_string(),
                        ))
                    })?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        let completed_at = if let Some(completed_str) = row.get::<Option<String>, _>("completed_at")
        {
            Some(
                DateTime::parse_from_rfc3339(&completed_str)
                    .map_err(|e| {
                        LuceError::IoError(io::Error::new(
                            io::ErrorKind::InvalidData,
                            e.to_string(),
                        ))
                    })?
                    .with_timezone(&Utc),
            )
        } else {
            None
        };

        Ok(Task {
            id: task_id,
            title: row.get("title"),
            description: row.get("description"),
            status,
            priority,
            assigned_session: row.get("assigned_session"),
            metadata,
            created_at,
            updated_at,
            started_at,
            completed_at,
        })
    }

    async fn delete_task(&self, id: TaskId) -> Result<(), LuceError> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))?;

        if result.rows_affected() == 0 {
            return Err(LuceError::TaskNotFound { id });
        }

        Ok(())
    }

    async fn list_tasks(&self) -> Result<Vec<Task>, LuceError> {
        let rows = sqlx::query("SELECT * FROM tasks ORDER BY created_at")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))?;

        let mut tasks = Vec::new();
        for row in rows {
            let task_id: TaskId = Uuid::parse_str(row.get("id")).map_err(|e| {
                LuceError::IoError(io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
            })?;

            let status: TaskStatus =
                serde_json::from_str(&format!("\"{}\"", row.get::<String, _>("status"))).map_err(
                    |e| {
                        LuceError::IoError(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Invalid status: {}", e),
                        ))
                    },
                )?;

            let priority: TaskPriority =
                serde_json::from_str(&format!("\"{}\"", row.get::<String, _>("priority")))
                    .map_err(|e| {
                        LuceError::IoError(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("Invalid priority: {}", e),
                        ))
                    })?;

            let metadata = Self::deserialize_metadata(row.get("metadata"))?;

            let created_at: DateTime<Utc> = DateTime::parse_from_rfc3339(row.get("created_at"))
                .map_err(|e| {
                    LuceError::IoError(io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
                })?
                .with_timezone(&Utc);

            let updated_at: DateTime<Utc> = DateTime::parse_from_rfc3339(row.get("updated_at"))
                .map_err(|e| {
                    LuceError::IoError(io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
                })?
                .with_timezone(&Utc);

            let started_at = if let Some(started_str) = row.get::<Option<String>, _>("started_at") {
                Some(
                    DateTime::parse_from_rfc3339(&started_str)
                        .map_err(|e| {
                            LuceError::IoError(io::Error::new(
                                io::ErrorKind::InvalidData,
                                e.to_string(),
                            ))
                        })?
                        .with_timezone(&Utc),
                )
            } else {
                None
            };

            let completed_at =
                if let Some(completed_str) = row.get::<Option<String>, _>("completed_at") {
                    Some(
                        DateTime::parse_from_rfc3339(&completed_str)
                            .map_err(|e| {
                                LuceError::IoError(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    e.to_string(),
                                ))
                            })?
                            .with_timezone(&Utc),
                    )
                } else {
                    None
                };

            tasks.push(Task {
                id: task_id,
                title: row.get("title"),
                description: row.get("description"),
                status,
                priority,
                assigned_session: row.get("assigned_session"),
                metadata,
                created_at,
                updated_at,
                started_at,
                completed_at,
            });
        }

        Ok(tasks)
    }
}

#[async_trait]
impl TaskRepository for &SqliteTaskRepository {
    async fn save_task(&self, task: &Task) -> Result<(), LuceError> {
        (*self).save_task(task).await
    }

    async fn get_task(&self, id: TaskId) -> Result<Task, LuceError> {
        (*self).get_task(id).await
    }

    async fn delete_task(&self, id: TaskId) -> Result<(), LuceError> {
        (*self).delete_task(id).await
    }

    async fn list_tasks(&self) -> Result<Vec<Task>, LuceError> {
        (*self).list_tasks().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luce_migrations::{MigrationApplier, MigrationRunner};
    use std::fs;
    use tempfile::{tempdir, NamedTempFile};

    async fn create_test_task_repo() -> SqliteTaskRepository {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().to_str().unwrap());

        // Set up migrations
        let applier = MigrationApplier::new(&db_url).await.unwrap();

        // Create a temporary migration with just the tasks table
        let temp_dir = tempdir().unwrap();
        let migrations_dir = temp_dir.path();

        fs::write(
            migrations_dir.join("20250320174208_create_tasks_table.sql"),
            r#"
            CREATE TABLE tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL,
                priority TEXT NOT NULL,
                assigned_session TEXT,
                metadata TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                started_at TEXT,
                completed_at TEXT
            );
            "#,
        )
        .unwrap();

        // Apply migrations
        applier.run_migrations(migrations_dir).await.unwrap();

        SqliteTaskRepository::new(&db_url).await.unwrap()
    }

    #[tokio::test]
    async fn test_task_repository_save_and_get() {
        let repo = create_test_task_repo().await;
        let task = Task::new("Test task".to_string());
        let task_id = task.id;

        repo.save_task(&task).await.unwrap();
        let retrieved_task = repo.get_task(task_id).await.unwrap();

        assert_eq!(task.id, retrieved_task.id);
        assert_eq!(task.title, retrieved_task.title);
        assert_eq!(task.status, retrieved_task.status);
    }

    #[tokio::test]
    async fn test_task_repository_list_tasks() {
        let repo = create_test_task_repo().await;
        let task1 = Task::new("Task 1".to_string());
        let task2 = Task::new("Task 2".to_string());

        repo.save_task(&task1).await.unwrap();
        repo.save_task(&task2).await.unwrap();

        let tasks = repo.list_tasks().await.unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_task_repository_delete() {
        let repo = create_test_task_repo().await;
        let task = Task::new("Test task".to_string());
        let task_id = task.id;

        repo.save_task(&task).await.unwrap();
        repo.delete_task(task_id).await.unwrap();

        let result = repo.get_task(task_id).await;
        assert!(matches!(result, Err(LuceError::TaskNotFound { .. })));
    }

    #[tokio::test]
    async fn test_task_with_metadata() {
        let repo = create_test_task_repo().await;
        let task = Task::new("Test task".to_string())
            .with_metadata("category".to_string(), "urgent".to_string())
            .with_metadata("assignee".to_string(), "alice".to_string());
        let task_id = task.id;

        repo.save_task(&task).await.unwrap();
        let retrieved_task = repo.get_task(task_id).await.unwrap();

        assert_eq!(
            retrieved_task.metadata.get("category"),
            Some(&"urgent".to_string())
        );
        assert_eq!(
            retrieved_task.metadata.get("assignee"),
            Some(&"alice".to_string())
        );
    }
}
