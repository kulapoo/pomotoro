use super::*;
use anyhow::Context;
use usecases::task::{SearchTasksQuery, search_tasks as search_tasks_usecase};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchTasksRequest {
    pub query: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[tauri::command(rename_all = "snake_case")]
pub async fn search_tasks(
    request: SearchTasksRequest,
    task_repo: State<'_, Arc<dyn TaskRepository + Send + Sync>>,
) -> Result<Vec<Task>, String> {
    let query = SearchTasksQuery {
        query: request.query,
        tags: request.tags,
        status: request.status,
        sort_by: request.sort_by,
        sort_order: request.sort_order,
        limit: request.limit,
        offset: request.offset,
    };

    search_tasks_usecase(&task_repo, query)
        .await
        .context("Failed to search tasks")
        .map_err(|e| e.to_string())
}
