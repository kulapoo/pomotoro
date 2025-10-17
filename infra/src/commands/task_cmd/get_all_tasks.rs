use super::*;
use anyhow::Context;
use usecases::task::{get_tasks, GetTasksQuery};

#[tauri::command(rename_all = "snake_case")]
pub async fn get_all_tasks(
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Vec<TaskDto>, String> {
    let query = GetTasksQuery {
        tags: None,
        status: None,
        active_only: false,
    };
    let tasks = get_tasks(&task_repo, query)
        .await
        .context("Failed to get all tasks")
        .map_err(|e| e.to_string())?;
    Ok(tasks.into_iter().map(TaskDto::from).collect())
}