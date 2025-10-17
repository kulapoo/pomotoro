use super::*;
use anyhow::{anyhow, Context};
use log::info;
use usecases::task::complete_session;

#[tauri::command(rename_all = "snake_case")]
pub async fn complete_task_session(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    event_publisher: State<'_, Arc<dyn EventPublisher + Send + Sync>>,
) -> Result<TaskDto, String> {
    info!("Completing task session: id={}", task_id);

    let task_id_parsed = TaskId::from_string(&task_id)
        .context("Invalid task ID")
        .map_err(|e| e.to_string())?;

    // Complete a session for the task
    complete_session(&task_repo, &event_publisher, task_id_parsed)
        .await
        .with_context(|| format!("Failed to complete session for task: {}", task_id))
        .map_err(|e| {
            log::error!("Failed to complete session for task {}: {}", task_id, e);
            e.to_string()
        })?;

    let task = task_repo
        .get_by_id(task_id_parsed)
        .await
        .context("Failed to retrieve task after completing session")
        .map_err(|e| e.to_string())?
        .ok_or_else(|| anyhow!("Task not found after completing session"))
        .map_err(|e| e.to_string())?;

    info!("Successfully completed session for task: id={}", task_id);
    Ok(task.into())
}