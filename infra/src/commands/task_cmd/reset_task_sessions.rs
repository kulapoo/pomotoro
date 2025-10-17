use super::*;
use anyhow::{anyhow, Context};
use log::info;
use usecases::task::reset_sessions;

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_task_sessions(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Task, String> {
    info!("Resetting sessions for task: id={}", task_id);

    let task_id_parsed =
        TaskId::from_string(&task_id).map_err(|_| format!("Invalid task ID: {}", task_id))?;

    reset_sessions(&task_repo, task_id_parsed)
        .await
        .with_context(|| format!("Failed to reset sessions for task: {}", task_id))
        .map_err(|e| {
            log::error!("Failed to reset sessions for task {}: {}", task_id, e);
            e.to_string()
        })?;

    let task = task_repo
        .get_by_id(task_id_parsed)
        .await
        .context("Failed to retrieve task after resetting sessions")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| anyhow!("Task not found after resetting sessions"))
        .map_err(|e| e.to_string())?;

    info!("Successfully reset sessions for task: id={}", task_id);
    Ok(task)
}