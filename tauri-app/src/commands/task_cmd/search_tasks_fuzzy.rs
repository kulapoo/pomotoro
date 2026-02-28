use super::*;
use anyhow::Context;
use usecases::task::search_tasks_fuzzy as search_tasks_fuzzy_usecase;

#[tauri::command(rename_all = "snake_case")]
pub async fn search_tasks_fuzzy(
    query: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Vec<Task>, String> {
    search_tasks_fuzzy_usecase(&task_repo, query)
        .await
        .context("Failed to perform fuzzy search")
        .map_err(|e| e.to_string())
}