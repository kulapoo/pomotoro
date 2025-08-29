use domain::{Error, Result, Task, TaskId, TaskRepository, TaskStatus};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct GetTaskQuery {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct GetTasksQuery {
    pub tags: Option<Vec<String>>,
    pub status: Option<TaskStatus>,
    pub active_only: bool,
}

pub async fn get_task(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    query: GetTaskQuery,
) -> Result<Task> {
    let task_id =
        TaskId::from_string(&query.id).map_err(|_| Error::TaskNotFound {
            id: query.id.clone(),
        })?;

    task_repo
        .get_by_id(task_id)
        .await?
        .ok_or(Error::TaskNotFound { id: query.id })
}

pub async fn get_tasks(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    query: GetTasksQuery,
) -> Result<Vec<Task>> {
    let tasks = if query.active_only {
        task_repo.get_active_tasks().await?
    } else {
        task_repo.get_all().await?
    };

    let mut filtered_tasks = tasks;

    if let Some(status) = query.status {
        filtered_tasks.retain(|task| task.status == status);
    }

    if let Some(tags) = query.tags {
        filtered_tasks
            .retain(|task| tags.iter().any(|tag| task.tags.contains(tag)));
    }

    Ok(filtered_tasks)
}

pub async fn get_task_by_tags(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    tags: Vec<String>,
) -> Result<Vec<Task>> {
    task_repo.get_by_tags(&tags).await
}

pub async fn get_tasks_by_status(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    status: TaskStatus,
) -> Result<Vec<Task>> {
    task_repo.get_by_status(status).await
}

