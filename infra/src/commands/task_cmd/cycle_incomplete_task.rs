use super::*;
use anyhow::Context;
use domain::{TaskId, TaskRepository};
use usecases::task::cycle_incomplete_task::{
    cycle_incomplete_task as cycle_task_pure, CycleDirection,
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
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<CycleIncompleteTaskResponse, String> {
    // Parse direction
    let direction = match request.direction.as_str() {
        "next" => CycleDirection::Next,
        "previous" => CycleDirection::Previous,
        _ => CycleDirection::Next,
    };

    // Parse task ID if provided
    let current_task_id = if let Some(id_str) = request.current_task_id {
        Some(
            TaskId::from_string(&id_str)
                .map_err(|_| format!("Invalid task ID: {}", id_str))?,
        )
    } else {
        None
    };

    // Fetch all tasks from repository (async I/O)
    let tasks = task_repo
        .get_all()
        .await
        .context("Failed to fetch tasks")
        .map_err(|e| e.to_string())?;

    // Call pure use case function
    let result = cycle_task_pure(
        &tasks,
        current_task_id.as_ref(),
        direction,
    );

    Ok(CycleIncompleteTaskResponse {
        task: result.task,
        position: result.position,
        total_incomplete: result.total_incomplete,
        has_more_tasks: result.has_more_tasks,
    })
}