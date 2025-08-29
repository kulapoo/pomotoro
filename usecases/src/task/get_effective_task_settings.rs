use domain::{
    ConfigRepository, EffectiveSettings, Result, TaskId, TaskRepository
};
use std::sync::Arc;

pub async fn get_effective_task_settings(
    task_repository: &Arc<dyn TaskRepository + Send + Sync>,
    config_repository: &Arc<dyn ConfigRepository + Send + Sync>,
    task_id: TaskId,
) -> Result<EffectiveSettings> {
    let task = task_repository.get_by_id(task_id).await?
        .ok_or(domain::Error::TaskNotFound { id: task_id.to_string() })?;
    let config = config_repository.get_config().await?;

    Ok(task.get_effective_settings(&config.task_defaults))
}

