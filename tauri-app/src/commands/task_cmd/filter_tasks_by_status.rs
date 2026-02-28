use super::*;
use anyhow::Context;
use usecases::task::{filter_tasks_by_status as filter_tasks_usecase, FilterTasksByStatusQuery};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterTasksRequest {
    pub status: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn filter_tasks_by_status(
    request: FilterTasksRequest,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Vec<Task>, String> {
    let query = FilterTasksByStatusQuery {
        status: request.status,
        limit: request.limit,
        offset: request.offset,
    };

    filter_tasks_usecase(&task_repo, query)
        .await
        .context("Failed to filter tasks by status")
        .map_err(|e| e.to_string())
}