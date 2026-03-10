use domain::{EventPublisher, Result, TaskCompleted, TaskId, TaskRepository};
use std::sync::Arc;

use crate::get_task_by_id;

pub async fn complete_task(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    task_id: TaskId,
) -> Result<()> {
    let mut task = get_task_by_id(task_repo, task_id).await?;

    // Mark all sessions as complete via domain method
    task.complete();

    task_repo.update(task.clone()).await?;

    let completed_event = TaskCompleted::new(
        task.id(),
        task.current_sessions(),
        task.current_sessions() as u64 + 1,
    );

    event_publisher.publish(Box::new(completed_event));

    Ok(())
}
