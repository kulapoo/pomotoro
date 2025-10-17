use super::*;
use anyhow::Context;
use usecases::task::cycle_incomplete_task::{
    cycle_incomplete_task as cycle_task_usecase, CycleDirection, CycleIncompleteTaskQuery,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleIncompleteTaskRequest {
    pub current_task_id: Option<String>,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleIncompleteTaskResponse {
    pub task: Option<Task>,
    pub position: usize,
    pub total_incomplete: usize,
    pub has_more_tasks: bool,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn cycle_incomplete_task(
    request: CycleIncompleteTaskRequest,
    cycling_service: State<'_, Arc<dyn domain::TaskCyclerService + Send + Sync>>,
) -> Result<CycleIncompleteTaskResponse, String> {
    let direction = match request.direction.as_str() {
        "next" => CycleDirection::Next,
        "previous" => CycleDirection::Previous,
        _ => CycleDirection::Next,
    };

    let query = CycleIncompleteTaskQuery {
        current_task_id: request.current_task_id,
        direction,
    };

    let result = cycle_task_usecase(&cycling_service, query)
        .await
        .context("Failed to cycle incomplete task")
        .map_err(|e| e.to_string())?;

    Ok(CycleIncompleteTaskResponse {
        task: result.task,
        position: result.position,
        total_incomplete: result.total_incomplete,
        has_more_tasks: result.has_more_tasks,
    })
}