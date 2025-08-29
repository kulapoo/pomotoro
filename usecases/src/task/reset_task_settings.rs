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

