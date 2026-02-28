use super::*;
use anyhow::Context;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_incomplete_tasks(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Vec<Task>, String> {
    task_repo
        .get_incomplete_tasks()
        .await
        .context("Failed to get incomplete tasks")
        .map_err(|e| e.to_string())
}