use super::*;
use anyhow::{anyhow, Context};
use log::info;
use usecases::task::reset_task as reset_task_usecase;

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_task(
    task_id: String,
    reset_sessions: bool,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<TaskDto, String> {
    info!(
        "Resetting task: id={}, reset_sessions={}",
        task_id, reset_sessions
    );

    let task_id_parsed = TaskId::from_string(&task_id).map_err(|e| {
        log::error!("Invalid task ID '{}': {}", task_id, e);
        format!("Invalid task ID: {}", task_id)
    })?;

    reset_task_usecase(&task_repo, task_id_parsed, reset_sessions)
        .await
        .with_context(|| format!("Failed to reset task: {}", task_id))
        .map_err(|e| {
            log::error!("Failed to reset task {}: {}", task_id, e);
            e.to_string()
        })?;

    let task = task_repo
        .get_by_id(task_id_parsed)
        .await
        .context("Failed to retrieve task after reset")
        .map_err(|e| {
            log::error!("Failed to retrieve task {} after reset: {}", task_id, e);
            e.to_string()
        })?
        .ok_or_else(|| {
            log::error!("Task not found after reset: {}", task_id);
            anyhow!("Task not found after reset")
        })
        .map_err(|e| e.to_string())?;

    info!(
        "Successfully reset task: id={}, new_status={:?}",
        task_id, task.status
    );
    Ok(TaskDto::from(task))
}