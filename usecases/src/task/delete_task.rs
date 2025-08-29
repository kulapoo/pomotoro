use domain::{Error, EventPublisher, Result, TaskId, TaskRepository};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DeleteTaskCmd {
    pub id: String,
}

pub async fn delete_task(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    _event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    cmd: DeleteTaskCmd,
) -> Result<bool> {
    let task_id = TaskId::from_string(&cmd.id)
        .map_err(|_| Error::TaskNotFound { id: cmd.id.clone() })?;

    let task = task_repo
        .get_by_id(task_id)
        .await?
        .ok_or_else(|| Error::TaskNotFound { id: cmd.id.clone() })?;

    if task.name == "Focus Session"
        && task.description
            == Some("Default pomodoro task for focused work".to_string())
    {
        return Err(Error::InvalidStateTransition {
            from: "default_task".to_string(),
            to: "deleted".to_string(),
        });
    }

    let deleted = task_repo.delete(task_id).await?;

    if deleted {
        // TODO: Publish TaskDeleted event - domain event not yet implemented
        // let deleted_event = TaskDeleted::new(task.id, task.name);
        // event_publisher.publish(Box::new(deleted_event));
    }

    Ok(deleted)
}

