use async_trait::async_trait;
use luce_shared::{LuceError, TaskDependency, TaskId};
use sqlx::{Row, SqlitePool};

use super::DependencyRepository;

pub struct SqliteDependencyRepository {
    pool: SqlitePool,
}

impl SqliteDependencyRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DependencyRepository for SqliteDependencyRepository {
    async fn save_dependency(&self, dependency: &TaskDependency) -> Result<(), LuceError> {
        sqlx::query(
            r#"
            INSERT INTO task_dependencies (task_id, depends_on_task_id, created_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(dependency.task_id.to_string())
        .bind(dependency.depends_on_task_id.to_string())
        .bind(dependency.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| LuceError::DatabaseError {
            message: e.to_string(),
        })?;

        Ok(())
    }

    async fn remove_dependency(
        &self,
        task_id: TaskId,
        depends_on_task_id: TaskId,
    ) -> Result<(), LuceError> {
        sqlx::query("DELETE FROM task_dependencies WHERE task_id = ? AND depends_on_task_id = ?")
            .bind(task_id.to_string())
            .bind(depends_on_task_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| LuceError::DatabaseError {
                message: e.to_string(),
            })?;

        Ok(())
    }

    async fn get_dependencies(&self, task_id: TaskId) -> Result<Vec<TaskId>, LuceError> {
        let rows =
            sqlx::query("SELECT depends_on_task_id FROM task_dependencies WHERE task_id = ?")
                .bind(task_id.to_string())
                .fetch_all(&self.pool)
                .await
                .map_err(|e| LuceError::DatabaseError {
                    message: e.to_string(),
                })?;

        Ok(rows
            .iter()
            .map(|row| {
                let id_str: String = row.get("depends_on_task_id");
                uuid::Uuid::parse_str(&id_str).unwrap()
            })
            .collect())
    }

    async fn get_dependents(&self, task_id: TaskId) -> Result<Vec<TaskId>, LuceError> {
        let rows =
            sqlx::query("SELECT task_id FROM task_dependencies WHERE depends_on_task_id = ?")
                .bind(task_id.to_string())
                .fetch_all(&self.pool)
                .await
                .map_err(|e| LuceError::DatabaseError {
                    message: e.to_string(),
                })?;

        Ok(rows
            .iter()
            .map(|row| {
                let id_str: String = row.get("task_id");
                uuid::Uuid::parse_str(&id_str).unwrap()
            })
            .collect())
    }

    async fn remove_all_dependencies(&self, task_id: TaskId) -> Result<(), LuceError> {
        sqlx::query("DELETE FROM task_dependencies WHERE task_id = ? OR depends_on_task_id = ?")
            .bind(task_id.to_string())
            .bind(task_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| LuceError::DatabaseError {
                message: e.to_string(),
            })?;

        Ok(())
    }
}
