use domain::{
    EventPublisher, Result, Task, TaskId, TaskRepository, TaskSettings, TaskUpdated
};
use std::sync::Arc;

pub async fn update_task_settings(
    repository: &Arc<dyn TaskRepository + Send + Sync>,
    publisher: &Arc<dyn EventPublisher + Send + Sync>,
    task_id: TaskId,
    settings: TaskSettings,
) -> Result<Task> {
    let mut task = repository.get_by_id(task_id).await?
        .ok_or(domain::Error::TaskNotFound { id: task_id.to_string() })?;

    task.set_settings(settings);

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
    async fn test_update_task_settings() {
        let repository: Arc<dyn TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::new());
        let publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(MockEventPublisher::new());

        let task = TaskBuilder::new()
            .name("Test Task".to_string())
            .max_sessions(4)
            .build()
            .unwrap();
        let task_id = task.id();
        repository.create(task).await.unwrap();

        let mut settings = TaskSettings::default();
        settings.use_global_settings = false;
        settings.max_sessions = Some(6);
        settings.work_duration = Some(Duration::from_secs(30 * 60));

        let updated_task = update_task_settings(
            &repository,
            &publisher,
            task_id,
            settings.clone()
        ).await.unwrap();

        assert!(updated_task.has_custom_settings());
        assert_eq!(
            updated_task.settings.max_sessions,
            Some(6)
        );
        assert_eq!(
            updated_task.settings.work_duration,
            Some(Duration::from_secs(30 * 60))
        );
    }
}