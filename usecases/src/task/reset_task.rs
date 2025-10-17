use domain::{Error, Result, TaskId, TaskRepository, TaskStatus, TaskPatch};
use std::sync::Arc;
use chrono::Utc;

/// Resets a completed task back to Queued status with optional session reset
pub async fn reset_task(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    task_id: TaskId,
    reset_sessions: bool,
) -> Result<()> {
    let mut task = task_repo.get_by_id(task_id).await?.ok_or_else(|| {
        Error::TaskNotFound {
            id: task_id.to_string(),
        }
    })?;

    // Allow resetting from any status, including Completed
    let patch = if reset_sessions {
        TaskPatch {
            status: Some(TaskStatus::Queued),
            current_sessions: Some(0),
            completed_at: None,
            updated_at: Some(Utc::now()),
            ..Default::default()
        }
    } else {
        TaskPatch {
            status: Some(TaskStatus::Queued),
            completed_at: None,
            updated_at: Some(Utc::now()),
            ..Default::default()
        }
    };

    task.patch(patch);
    task_repo.update(task).await
}