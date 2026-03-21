pub mod luce_service;

pub use luce_service::LuceService;

#[cfg(test)]
mod tests {
    use super::*;
    use luce_shared::TaskPriority;
    use tempfile::NamedTempFile;

    async fn create_test_service() -> LuceService {
        let temp_file = NamedTempFile::new().unwrap();
        LuceService::with_database(temp_file.path()).await.unwrap()
    }

    #[tokio::test]
    async fn test_service_create_and_get_task() {
        let service = create_test_service().await;

        let task = service
            .create_task(
                "Test task".to_string(),
                Some("Test description".to_string()),
                Some(TaskPriority::High),
            )
            .await
            .unwrap();

        let retrieved_task = service.get_task(&task.id.to_string()).await.unwrap();

        assert_eq!(task.id, retrieved_task.id);
        assert_eq!(task.title, retrieved_task.title);
        assert_eq!(task.priority, retrieved_task.priority);
    }

    #[tokio::test]
    async fn test_service_list_tasks() {
        let service = create_test_service().await;

        // Create a few tasks
        service
            .create_task("Task 1".to_string(), None, None)
            .await
            .unwrap();
        service
            .create_task("Task 2".to_string(), None, None)
            .await
            .unwrap();

        let tasks = service.list_tasks(None).await.unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_service_update_task_status() {
        let service = create_test_service().await;

        let task = service
            .create_task("Test task".to_string(), None, None)
            .await
            .unwrap();

        service
            .update_task_status(&task.id.to_string(), luce_shared::TaskStatus::Completed)
            .await
            .unwrap();

        let updated_task = service.get_task(&task.id.to_string()).await.unwrap();
        assert_eq!(updated_task.status, luce_shared::TaskStatus::Completed);
    }
}
