use super::*;
use anyhow::Context;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_active_task(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
    timer_repo: State<'_, Arc<dyn TimerRepository + Send + Sync>>,
) -> Result<Option<Task>, String> {
    let timer = timer_repo
        .get()
        .await
        .context("Failed to retrieve timer state")
        .map_err(|e| e.to_string())?;

    let Some(task_id) = timer.task_id() else {
        return Ok(None);
    };

    let task = task_repo
        .get_by_id(task_id)
        .await
        .context("Failed to get active task")
        .map_err(|e| e.to_string())?;

    Ok(task)
}
