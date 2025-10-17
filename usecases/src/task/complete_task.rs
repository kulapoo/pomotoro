use domain::{
    Error, EventPublisher, Result, TaskCompleted, TaskId, TaskRepository,
    TaskSessionCompleted,
};
use std::sync::Arc;

use crate::get_task_by_id;

pub async fn complete_task(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    task_id: TaskId,
) -> Result<()> {
    let mut task = get_task_by_id(
        task_repo,
        task_id,
    )
    .await?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    // Mark all sessions as complete
    task.current_sessions = task.max_sessions;
    task.status = domain::TaskStatus::Completed;

    task_repo.update(task.clone()).await?;

    let session_event = TaskSessionCompleted::new(
        task.id,
        task.current_sessions,
        task.max_sessions,
        true, // Task is always completed when this function is called
        task.current_sessions as u64,
    );

    event_publisher.publish(Box::new(session_event));

    let completed_event = TaskCompleted::new(
        task.id,
        task.current_sessions,
        task.current_sessions as u64 + 1,
    );

    event_publisher.publish(Box::new(completed_event));

    Ok(())
}
