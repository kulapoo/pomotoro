#[cfg(test)]
mod file_storage_integration_tests {
    use domain::{Task, TaskRepository};
    use domain::timer::TimerService;
    use infra::adapters::{
        FileTaskRepository, FileTimerService, FileStorageService, StorageConfig,
        InMemoryEventBus, InMemoryConfigRepository,
    };
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio;

    #[tokio::test]
    async fn test_file_task_repository_persistence() {
        let temp_dir = tempdir().unwrap();
        let tasks_file = temp_dir.path().join("tasks.json");
        
        let repo = FileTaskRepository::new(tasks_file.clone());
        
        let task = Task::new("Test Task".to_string(), 4).unwrap();
        let task_id = task.id;
        
        repo.create(task).await.unwrap();
        
        let repo2 = FileTaskRepository::new(tasks_file);
        let loaded_task = repo2.get_by_id(task_id).await.unwrap();
        
        assert!(loaded_task.is_some());
        assert_eq!(loaded_task.unwrap().name, "Test Task");
    }

    #[tokio::test]
    async fn test_file_timer_service_persistence() {
        let temp_dir = tempdir().unwrap();
        let storage_path = Some(temp_dir.path().to_path_buf());
        
        let event_bus = Arc::new(InMemoryEventBus::new());
        let config_repository = Arc::new(InMemoryConfigRepository::new());
        let timer_service = FileTimerService::new(event_bus.clone(), storage_path.clone(), config_repository.clone());
        
        let task = Task::new("Timer Task".to_string(), 4).unwrap();
        timer_service.switch_task(task.id, Some(&task)).await.unwrap();
        
        let state1 = timer_service.get_state().await.unwrap();
        
        let timer_service2 = FileTimerService::new(event_bus, storage_path, config_repository);
        timer_service2.load_state().await.unwrap();
        
        let state2 = timer_service2.get_state().await.unwrap();
        
        assert_eq!(state1.active_entity_id(), state2.active_entity_id());
    }

    #[tokio::test]
    async fn test_storage_service_export_import() {
        let temp_dir = tempdir().unwrap();
        let storage_config = StorageConfig {
            storage_location: infra::adapters::StorageLocation::Custom(temp_dir.path().to_path_buf()),
            auto_save_interval_seconds: 60,
            backup_enabled: true,
            max_backup_count: 5,
        };
        
        let storage_service = FileStorageService::new(storage_config).unwrap();
        
        let tasks_file = temp_dir.path().join("tasks.json");
        let task = Task::new("Export Task".to_string(), 2).unwrap();
        let repo = FileTaskRepository::new(tasks_file);
        repo.create(task).await.unwrap();
        
        let export_data = storage_service.export_data().await.unwrap();
        assert!(!export_data.tasks.is_null());
        
        let backup_info = storage_service.create_backup().await.unwrap();
        assert!(backup_info.path.exists());
        
        storage_service.restore_backup(backup_info.id).await.unwrap();
    }

    #[tokio::test]
    async fn test_timer_state_recovery_after_crash() {
        let temp_dir = tempdir().unwrap();
        let storage_path = Some(temp_dir.path().to_path_buf());
        
        let event_bus = Arc::new(InMemoryEventBus::new());
        let config_repository = Arc::new(InMemoryConfigRepository::new());
        let timer_service = FileTimerService::new(event_bus.clone(), storage_path.clone(), config_repository.clone());
        
        let task = Task::new("Crash Test Task".to_string(), 4).unwrap();
        timer_service.switch_task(task.id, Some(&task)).await.unwrap();
        timer_service.start_timer(Some(&task)).await.unwrap();
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        timer_service.toggle_pause().await.unwrap();
        let paused_state = timer_service.get_state().await.unwrap();
        
        drop(timer_service);
        
        let timer_service2 = FileTimerService::new(event_bus, storage_path, config_repository);
        timer_service2.load_state().await.unwrap();
        
        let recovered_state = timer_service2.get_state().await.unwrap();
        
        assert!(recovered_state.is_paused());
        assert_eq!(paused_state.active_entity_id(), recovered_state.active_entity_id());
    }
}