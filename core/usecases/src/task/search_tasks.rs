use domain::task::repository::{
    SearchCriteria, SearchOptions, SortBy, SortOrder,
};
use domain::{Error, Result, Task, TaskRepository, TaskStatus};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchTasksQuery {
    pub query: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn search_tasks(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    query: SearchTasksQuery,
) -> Result<Vec<Task>> {
    let mut criteria = SearchCriteria::new()
        .with_limit(query.limit.unwrap_or(100))
        .with_offset(query.offset.unwrap_or(0));

    if let Some(q) = query.query.filter(|s| !s.is_empty()) {
        criteria = criteria.with_query(q);
    }

    if let Some(tags) = query.tags.filter(|t| !t.is_empty()) {
        criteria = criteria.with_tags(tags);
    }

    if let Some(status) = query.status.filter(|s| !s.is_empty()) {
        criteria = criteria.with_status(status);
    }

    let sort_by = query.sort_by.as_ref().and_then(|s| match s.as_str() {
        "name" => Some(SortBy::Name),
        "created_at" => Some(SortBy::CreatedAt),
        "sessions_completed" => Some(SortBy::SessionsCompleted),
        "status" => Some(SortBy::Status),
        _ => None,
    });

    let sort_order = query.sort_order.as_ref().and_then(|s| match s.as_str() {
        "asc" | "ascending" => Some(SortOrder::Ascending),
        "desc" | "descending" => Some(SortOrder::Descending),
        _ => None,
    });

    let options = SearchOptions {
        criteria,
        sort_by,
        sort_order,
    };

    task_repo
        .search(options)
        .await
        .map_err(|e| Error::RepositoryError {
            message: format!("Failed to search tasks: {}", e),
        })
}

pub async fn search_tasks_fuzzy(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    query: String,
) -> Result<Vec<Task>> {
    task_repo
        .search_fuzzy(&query)
        .await
        .map_err(|e| Error::RepositoryError {
            message: format!("Failed to perform fuzzy search: {}", e),
        })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterTasksByStatusQuery {
    pub status: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

pub async fn filter_tasks_by_status(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    query: FilterTasksByStatusQuery,
) -> Result<Vec<Task>> {
    let status = match query.status.to_lowercase().as_str() {
        "active" => TaskStatus::Active,
        "completed" => TaskStatus::Completed,
        "paused" => TaskStatus::Paused,
        "queued" => TaskStatus::Queued,
        _ => {
            return Err(Error::ConfigurationError {
                message: format!("Invalid task status: {}", query.status),
            });
        }
    };

    let mut tasks = task_repo.get_by_status(status).await.map_err(|e| {
        Error::RepositoryError {
            message: format!("Failed to filter tasks by status: {}", e),
        }
    })?;

    if let Some(offset) = query.offset {
        tasks = tasks.into_iter().skip(offset).collect();
    }

    if let Some(limit) = query.limit {
        tasks = tasks.into_iter().take(limit).collect();
    }

    Ok(tasks)
}
