use domain::{Error, Result, TaskId, TaskPatch, TaskRepository, TaskStatus, TimerRepository, EventPublisher};
use std::sync::Arc;
use chrono::Utc;

use crate::timer::reset_timer_phase;


/// Resets a completed task back to Queued status with optional session reset
pub async fn reset_task(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    task_id: TaskId,
) -> Result<()> {
    let mut task = task_repo.get_by_id(task_id).await?.ok_or_else(|| {
        Error::TaskNotFound {
            id: task_id.to_string(),
        }
    })?;

    reset_timer_phase(task_id, task_repo.clone(), timer_repo.clone(), event_publisher.clone()).await?;

    // Allow resetting from any status, including Completed
    let patch = TaskPatch {
        status: Some(TaskStatus::Queued),
        current_sessions: Some(0),
        completed_at: None,
        updated_at: Some(Utc::now()),
        ..Default::default()
    };

    task.patch(patch);
    task_repo.update(task).await
}