use super::*;
use anyhow::Context;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_tasks_by_tags(
    tags: Vec<String>,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Vec<Task>, String> {
    task_repo
        .get_by_tags(&tags)
        .await
        .with_context(|| format!("Failed to get tasks with tags: {:?}", tags))
        .map_err(|e| e.to_string())
}
