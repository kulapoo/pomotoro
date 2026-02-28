use super::*;
use anyhow::Context;
use usecases::task::get_task_by_id;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_task(
    id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Option<Task>, String> {
    let task_id = TaskId::from_string(&id).map_err(|_| format!("Invalid task ID: {}", id))?;

    let result = get_task_by_id(&task_repo, task_id)
        .await
        .with_context(|| format!("Failed to get task with id: {}", id))
        .map_err(|e| e.to_string())?;
    Ok(Some(result))
}