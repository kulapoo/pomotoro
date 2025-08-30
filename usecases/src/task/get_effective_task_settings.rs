use domain::{
    EffectiveSettings, Result, TaskId, TaskRepository
};
use std::sync::Arc;

pub async fn get_effective_task_settings(
    task_repository: &Arc<dyn TaskRepository + Send + Sync>,
    task_id: TaskId,
) -> Result<EffectiveSettings> {
    let task = task_repository.get_by_id(task_id).await?
        .ok_or(domain::Error::TaskNotFound { id: task_id.to_string() })?;

    Ok(task.get_effective_settings())
}

