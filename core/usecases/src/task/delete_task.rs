use domain::{
    Error, EventPublisher, Result, TaskId, TaskRepository, TimerRepository,
    task::TaskDeleted,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DeleteTaskCmd {
    pub id: TaskId,
}

/// Delete a task by ID.
///
/// If the deleted task was the timer's active task, the timer is reset
/// to idle with no active task. The UI is expected to prompt the user
/// to select a new task.
pub async fn delete_task(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: DeleteTaskCmd,
) -> Result<bool> {
    let task = task_repo.get_by_id(cmd.id).await?.ok_or_else(|| {
        Error::TaskNotFound {
            id: cmd.id.to_string(),
        }
    })?;

    let deleted = task_repo.delete(cmd.id).await?;

    if deleted {
        // If we just deleted the timer's active task, detach it from
        // the timer so the UI can prompt for a new selection.
        let mut timer = timer_repo.get().await?;
        if timer.task_id() == Some(cmd.id) {
            timer.clear_task_id();
            timer_repo.save(&timer).await?;
        }

        let deleted_event = TaskDeleted::new(task.id(), 1);
        event_publisher.publish(Box::new(deleted_event));
    }

    Ok(deleted)
}
