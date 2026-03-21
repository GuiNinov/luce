use async_trait::async_trait;
use chrono::Utc;
use serde_json;
use sqlx::{Row, SqlitePool};
use std::io;
use uuid::Uuid;

use super::GraphRepository;
use luce_shared::{LuceError, TaskGraph};

pub struct SqliteGraphRepository {
    pool: SqlitePool,
}

impl SqliteGraphRepository {
    pub async fn new(database_url: &str) -> Result<Self, LuceError> {
        let pool = SqlitePool::connect(database_url)
            .await
            .map_err(|e| LuceError::IoError(io::Error::new(io::ErrorKind::Other, e.to_string())))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl GraphRepository for SqliteGraphRepository {
    async fn save_graph(&self, graph: &TaskGraph, id: &str) -> Result<(), LuceError> {
        let graph_json = serde_json::to_string(graph).map_err(LuceError::SerializationError)?;

        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO task_graphs (id, graph_data, created_at, updated_at)
            VALUES (?, ?, 
                COALESCE((SELECT created_at FROM task_graphs WHERE id = ?), ?),
                ?
            )
            "#,
        )
        .bind(id)
        .bind(graph_json)
        .bind(id)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| LuceError::IoError(io::Error::new(io::ErrorKind::Other, e.to_string())))?;

        Ok(())
    }

    async fn load_graph(&self, id: &str) -> Result<TaskGraph, LuceError> {
        let row = sqlx::query("SELECT graph_data FROM task_graphs WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => LuceError::TaskNotFound { id: Uuid::nil() }, // Using nil UUID for graph not found
                _ => LuceError::IoError(io::Error::new(io::ErrorKind::Other, e.to_string())),
            })?;

        let graph_json: String = row.get("graph_data");
        let graph: TaskGraph =
            serde_json::from_str(&graph_json).map_err(LuceError::SerializationError)?;

        Ok(graph)
    }

    async fn delete_graph(&self, id: &str) -> Result<(), LuceError> {
        let result = sqlx::query("DELETE FROM task_graphs WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| LuceError::IoError(io::Error::new(io::ErrorKind::Other, e.to_string())))?;

        if result.rows_affected() == 0 {
            return Err(LuceError::TaskNotFound { id: Uuid::nil() }); // Using nil UUID for graph not found
        }

        Ok(())
    }

    async fn list_graphs(&self) -> Result<Vec<String>, LuceError> {
        let rows = sqlx::query("SELECT id FROM task_graphs ORDER BY created_at")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| LuceError::IoError(io::Error::new(io::ErrorKind::Other, e.to_string())))?;

        let graph_ids = rows
            .into_iter()
            .map(|row| row.get::<String, _>("id"))
            .collect();

        Ok(graph_ids)
    }

    async fn graph_exists(&self, id: &str) -> Result<bool, LuceError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM task_graphs WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| LuceError::IoError(io::Error::new(io::ErrorKind::Other, e.to_string())))?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use luce_migrations::{MigrationApplier, MigrationRunner};
    use luce_shared::Task;
    use std::fs;
    use tempfile::{tempdir, NamedTempFile};

    async fn create_test_graph_repo() -> SqliteGraphRepository {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().to_str().unwrap());

        // Set up migrations
        let applier = MigrationApplier::new(&db_url).await.unwrap();

        // Create a temporary migration with just the task_graphs table
        let temp_dir = tempdir().unwrap();
        let migrations_dir = temp_dir.path();

        fs::write(
            migrations_dir.join("20250320174208_create_task_graphs_table.sql"),
            r#"
            CREATE TABLE task_graphs (
                id TEXT PRIMARY KEY,
                graph_data TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )
        .unwrap();

        // Apply migrations
        applier.run_migrations(migrations_dir).await.unwrap();

        SqliteGraphRepository::new(&db_url).await.unwrap()
    }

    #[tokio::test]
    async fn test_graph_repository_save_and_load() {
        let repo = create_test_graph_repo().await;
        let mut graph = TaskGraph::new();
        let task = Task::new("Test task".to_string());
        graph.add_task(task).unwrap();

        let graph_id = "test_graph";
        repo.save_graph(&graph, graph_id).await.unwrap();

        let retrieved_graph = repo.load_graph(graph_id).await.unwrap();
        assert_eq!(graph.tasks.len(), retrieved_graph.tasks.len());
    }

    #[tokio::test]
    async fn test_graph_repository_list() {
        let repo = create_test_graph_repo().await;
        let graph1 = TaskGraph::new();
        let graph2 = TaskGraph::new();

        repo.save_graph(&graph1, "graph1").await.unwrap();
        repo.save_graph(&graph2, "graph2").await.unwrap();

        let graphs = repo.list_graphs().await.unwrap();
        assert_eq!(graphs.len(), 2);
        assert!(graphs.contains(&"graph1".to_string()));
        assert!(graphs.contains(&"graph2".to_string()));
    }

    #[tokio::test]
    async fn test_graph_exists() {
        let repo = create_test_graph_repo().await;
        let graph = TaskGraph::new();

        assert!(!repo.graph_exists("nonexistent").await.unwrap());

        repo.save_graph(&graph, "test_graph").await.unwrap();
        assert!(repo.graph_exists("test_graph").await.unwrap());
    }
}
