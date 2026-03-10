use domain::{
    Error, EventPublisher, Result, Task, TaskId, TaskRepository, TaskUpdated,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SetDefaultTaskCmd {
    pub task_id: TaskId,
}

pub async fn set_default_task(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    cmd: SetDefaultTaskCmd,
) -> Result<Task> {
    let mut task =
        task_repo.get_by_id(cmd.task_id).await?.ok_or_else(|| {
            Error::TaskNotFound {
                id: cmd.task_id.to_string(),
            }
        })?;

    if task.is_default() {
        return Ok(task);
    }

    if let Some(mut current_default) = task_repo.get_default_task().await? {
        current_default.unset_as_default();
        task_repo.update(current_default.clone()).await?;

        // Publish event for the previously default task
        let updated_event =
            TaskUpdated::new(current_default.id(), None, None, None, None, 1);
        event_publisher.publish(Box::new(updated_event));
    }

    task.set_as_default();
    task_repo.update(task.clone()).await?;

    // Publish event for the newly default task
    let updated_event = TaskUpdated::new(task.id(), None, None, None, None, 1);
    event_publisher.publish(Box::new(updated_event));

    Ok(task)
}

pub async fn get_default_task(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
) -> Result<Option<Task>> {
    task_repo.get_default_task().await
}
