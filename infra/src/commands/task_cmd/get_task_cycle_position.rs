use super::*;
use anyhow::Context;
use domain::{TaskId, TaskRepository};
use usecases::task::cycle_incomplete_task::get_task_cycle_position as get_position_pure;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_task_cycle_position(
    task_id: String,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<(usize, usize), String> {
    // Parse task ID
    let task_id = TaskId::from_string(&task_id)
        .map_err(|_| format!("Invalid task ID: {}", task_id))?;

    // Fetch all tasks from repository (async I/O)
    let tasks = task_repo
        .get_all()
        .await
        .context("Failed to fetch tasks")
        .map_err(|e| e.to_string())?;

    // Call pure use case function
    let position = get_position_pure(&tasks, &task_id);

    Ok(position)
}