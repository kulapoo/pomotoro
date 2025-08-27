use domain::{
    ConfigRepository, EffectiveSettings, Result, TaskId, TaskRepository
};
use std::sync::Arc;

pub async fn get_effective_task_settings(
    task_repository: &Arc<dyn TaskRepository + Send + Sync>,
    config_repository: &Arc<dyn ConfigRepository + Send + Sync>,
    task_id: TaskId,
) -> Result<EffectiveSettings> {
    let task = task_repository.get_by_id(task_id).await?
        .ok_or(domain::Error::TaskNotFound { id: task_id.to_string() })?;
    let config = config_repository.get_config().await?;
    
    Ok(task.get_effective_settings(&config.task_defaults))
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        Config, TaskBuilder, TaskSettings, InMemoryConfigRepository, InMemoryTaskRepository,
    };
    use std::time::Duration;

    #[tokio::test]
    async fn test_get_effective_settings_with_global() {
        let task_repository: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        let config_repository: Arc<dyn ConfigRepository + Send + Sync> = Arc::new(InMemoryConfigRepository::new());
        
        let config = Config::default();
        config_repository.save_config(&config).await.unwrap();
        
        let task = TaskBuilder::new()
            .name("Test Task".to_string())
            .max_sessions(4)
            .build()
            .unwrap();
        let task_id = task.id();
        task_repository.create(task).await.unwrap();
        
        let settings = get_effective_task_settings(
            &task_repository,
            &config_repository,
            task_id
        ).await.unwrap();
        
        assert_eq!(settings.work_duration, config.task_defaults.work_duration);
        assert_eq!(settings.short_break_duration, config.task_defaults.short_break_duration);
        assert_eq!(settings.long_break_duration, config.task_defaults.long_break_duration);
    }

    #[tokio::test]
    async fn test_get_effective_settings_with_custom() {
        let task_repository: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        let config_repository: Arc<dyn ConfigRepository + Send + Sync> = Arc::new(InMemoryConfigRepository::new());
        
        let config = Config::default();
        config_repository.save_config(&config).await.unwrap();
        
        let custom_work_duration = Duration::from_secs(30 * 60);
        let mut task_settings = TaskSettings::default();
        task_settings.use_global_settings = false;
        task_settings.custom_work_duration = Some(custom_work_duration);
        
        let task = TaskBuilder::new()
            .name("Test Task".to_string())
            .max_sessions(4)
            .settings(task_settings)
            .build()
            .unwrap();
        let task_id = task.id();
        task_repository.create(task).await.unwrap();
        
        let settings = get_effective_task_settings(
            &task_repository,
            &config_repository,
            task_id
        ).await.unwrap();
        
        assert_eq!(settings.work_duration, custom_work_duration);
    }
}