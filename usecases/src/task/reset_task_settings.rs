use domain::{
    EventPublisher, Result, Task, TaskId, TaskRepository, TaskUpdated
};
use std::sync::Arc;

pub async fn reset_task_settings_to_defaults(
    repository: &Arc<dyn TaskRepository + Send + Sync>,
    publisher: &Arc<dyn EventPublisher + Send + Sync>,
    task_id: TaskId,
) -> Result<Task> {
    let mut task = repository.get_by_id(task_id).await?
        .ok_or(domain::Error::TaskNotFound { id: task_id.to_string() })?;

    task.reset_settings_to_global();

    repository.update(task.clone()).await?;

    publisher
        .publish(Box::new(TaskUpdated::new(
            task_id,
            Some(task.name.clone()),
            task.description.clone(),
            Some(task.max_sessions),
            Some(task.tags.clone()),
            0,
        )));

    Ok(task)
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::{
        MockEventPublisher, TaskBuilder, TaskSettings, InMemoryTaskRepository,
    };
    use std::time::Duration;

    #[tokio::test]
    async fn test_reset_task_settings_to_defaults() {
        let repository: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        let publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(MockEventPublisher::new());

        let task = TaskBuilder::new()
            .name("Test Task".to_string())
            .max_sessions(4)
            .settings(TaskSettings {
                use_global_settings: false,
                custom_max_sessions: Some(6),
                custom_work_duration: Some(Duration::from_secs(30 * 60)),
                custom_short_break_duration: None,
                custom_long_break_duration: None,
                custom_sessions_until_long_break: None,
                custom_enable_screen_blocking: None,
                custom_audio_config: None,
                custom_notification_config: None,
            })
            .build()
            .unwrap();
        let task_id = task.id();
        repository.create(task).await.unwrap();

        let updated_task = reset_task_settings_to_defaults(
            &repository,
            &publisher,
            task_id
        ).await.unwrap();

        assert!(!updated_task.has_custom_settings());
        assert!(updated_task.settings.use_global_settings);
    }
}