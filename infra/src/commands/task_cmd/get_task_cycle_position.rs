use super::*;
use anyhow::Context;
use usecases::task::get_task_cycle_position as get_position_usecase;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_task_cycle_position(
    task_id: String,
    cycling_service: State<'_, Arc<dyn domain::TaskCyclerService + Send + Sync>>,
) -> Result<(usize, usize), String> {
    get_position_usecase(&cycling_service, task_id)
        .await
        .context("Failed to get task cycle position")
        .map_err(|e| e.to_string())
}