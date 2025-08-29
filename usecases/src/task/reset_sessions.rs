use domain::{Error, Result, TaskId, TaskRepository};
use std::sync::Arc;

pub async fn reset_sessions(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    task_id: &str,
) -> Result<()> {
    let task_id =
        TaskId::from_string(task_id).map_err(|_| Error::TaskNotFound {
            id: task_id.to_string(),
        })?;

    let mut task = task_repo.get_by_id(task_id).await?.ok_or_else(|| {
        Error::TaskNotFound {
            id: task_id.to_string(),
        }
    })?;

    task.reset_sessions();
    task_repo.update(task).await
}

