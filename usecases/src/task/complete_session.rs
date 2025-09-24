use domain::{
    Error, EventPublisher, Result, TaskCompleted, TaskId, TaskRepository,
    TaskSessionCompleted,
};
use std::sync::Arc;

use crate::get_task_by_id;

#[derive(Debug, Clone)]
pub struct SessionCompletionResult {
    pub task_completed: bool,
    pub sessions_completed: u8,
    pub total_sessions: u8,
}

pub async fn complete_session(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    task_id: TaskId,
) -> Result<SessionCompletionResult> {
    let mut task = get_task_by_id(
        task_repo,
        task_id,
    )
    .await?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    task.increment_session()?;

    let is_task_completed = task.is_completed();

    task_repo.update(task.clone()).await?;

    let session_event = TaskSessionCompleted::new(
        task.id,
        task.current_sessions,
        task.max_sessions,
        is_task_completed,
        task.current_sessions as u64,
    );

    event_publisher.publish(Box::new(session_event));

    if is_task_completed {
        let completed_event = TaskCompleted::new(
            task.id,
            task.current_sessions,
            task.current_sessions as u64 + 1,
        );

        event_publisher.publish(Box::new(completed_event));
    }

    Ok(SessionCompletionResult {
        task_completed: is_task_completed,
        sessions_completed: task.current_sessions,
        total_sessions: task.max_sessions,
    })
}
