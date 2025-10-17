use super::*;
use anyhow::Context;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_active_tasks(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Vec<TaskDto>, String> {
    let tasks = task_repo
        .get_active_tasks()
        .await
        .context("Failed to get active tasks")
        .map_err(|e| e.to_string())?;
    Ok(tasks.into_iter().map(TaskDto::from).collect())
}