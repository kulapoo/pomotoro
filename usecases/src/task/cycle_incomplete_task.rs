use domain::{Error, Result, Task, TaskCyclerService, TaskId};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CycleIncompleteTaskQuery {
    pub current_task_id: Option<String>,
    pub direction: CycleDirection,
}

#[derive(Debug, Clone)]
pub enum CycleDirection {
    Next,
    Previous,
}

#[derive(Debug, Clone)]
pub struct IncompleteCycleResult {
    pub task: Option<Task>,
    pub position: usize,
    pub total_incomplete: usize,
    pub has_more_tasks: bool,
}

pub async fn cycle_incomplete_task(
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
    query: CycleIncompleteTaskQuery,
) -> Result<IncompleteCycleResult> {
    let current_id = if let Some(id_str) = query.current_task_id {
        Some(
            TaskId::from_string(&id_str)
                .map_err(|_| Error::TaskNotFound { id: id_str })?,
        )
    } else {
        None
    };

    let task = match query.direction {
        CycleDirection::Next => {
            cycling_service
                .cycle_to_next_incomplete_task(current_id)
                .await?
        }
        CycleDirection::Previous => {
            cycling_service
                .cycle_to_previous_incomplete_task(current_id)
                .await?
        }
    };

    let incomplete_queue = cycling_service.get_incomplete_task_queue().await?;
    let total_incomplete = incomplete_queue.len();

    let position = if let Some(ref current_task) = task {
        incomplete_queue
            .iter()
            .position(|t| t.id == current_task.id)
            .map(|p| p + 1)
            .unwrap_or(0)
    } else {
        0
    };

    let has_more_tasks = total_incomplete > 1;

    Ok(IncompleteCycleResult {
        task,
        position,
        total_incomplete,
        has_more_tasks,
    })
}

pub async fn get_incomplete_task_info(
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
    current_task_id: Option<String>,
) -> Result<IncompleteCycleResult> {
    let current_id = if let Some(id_str) = current_task_id {
        Some(
            TaskId::from_string(&id_str)
                .map_err(|_| Error::TaskNotFound { id: id_str })?,
        )
    } else {
        None
    };

    let incomplete_queue = cycling_service.get_incomplete_task_queue().await?;
    let total_incomplete = incomplete_queue.len();

    let current_task = if let Some(id) = current_id {
        incomplete_queue.iter().find(|t| t.id == id).cloned()
    } else {
        None
    };

    let position = if let Some(ref task) = current_task {
        incomplete_queue
            .iter()
            .position(|t| t.id == task.id)
            .map(|p| p + 1)
            .unwrap_or(0)
    } else {
        0
    };

    let has_more_tasks = total_incomplete > 1;

    Ok(IncompleteCycleResult {
        task: current_task,
        position,
        total_incomplete,
        has_more_tasks,
    })
}

pub async fn get_task_cycle_position(
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
    task_id: String,
) -> Result<(usize, usize)> {
    let id = TaskId::from_string(&task_id)
        .map_err(|_| Error::TaskNotFound { id: task_id })?;
    
    cycling_service.get_task_cycle_position(id).await
}