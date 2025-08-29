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

