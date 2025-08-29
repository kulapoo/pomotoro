use domain::{Error, Result, Task, TaskCyclerService, TaskId};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct GetNextTaskQuery {
    pub current_task_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TaskCycleResult {
    pub next_task: Option<Task>,
    pub has_more_tasks: bool,
    pub cycle_position: usize,
    pub total_tasks: usize,
}

pub async fn get_next_task(
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
    query: GetNextTaskQuery,
) -> Result<Option<Task>> {
    let current_task_id = if let Some(id_str) = query.current_task_id {
        Some(
            TaskId::from_string(&id_str)
                .map_err(|_| Error::TaskNotFound { id: id_str })?,
        )
    } else {
        None
    };

    cycling_service.get_next_task(current_task_id).await
}

pub async fn cycle_to_next_task(
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
    current_task_id: Option<String>,
) -> Result<TaskCycleResult> {
    let current_id = if let Some(id_str) = current_task_id {
        Some(
            TaskId::from_string(&id_str)
                .map_err(|_| Error::TaskNotFound { id: id_str })?,
        )
    } else {
        None
    };

    let next_task = cycling_service
        .cycle_to_next_active_task(current_id)
        .await?;

    let active_tasks = cycling_service.get_active_task_queue().await?;
    let total_tasks = active_tasks.len();

    let cycle_position =
        if let (Some(next), Some(_current)) = (&next_task, &current_id) {
            active_tasks
                .iter()
                .position(|t| t.id == next.id)
                .unwrap_or(0)
        } else {
            0
        };

    let has_more_tasks = total_tasks > 1;

    Ok(TaskCycleResult {
        next_task,
        has_more_tasks,
        cycle_position,
        total_tasks,
    })
}

pub async fn get_task_cycle_info(
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
    current_task_id: Option<String>,
) -> Result<TaskCycleResult> {
    let current_id = if let Some(id_str) = current_task_id {
        Some(
            TaskId::from_string(&id_str)
                .map_err(|_| Error::TaskNotFound { id: id_str })?,
        )
    } else {
        None
    };

    let active_tasks = cycling_service.get_active_task_queue().await?;
    let total_tasks = active_tasks.len();

    let current_task = if let Some(current_id) = current_id {
        active_tasks.iter().find(|t| t.id == current_id).cloned()
    } else {
        None
    };

    let cycle_position = if let Some(current) = &current_task {
        active_tasks
            .iter()
            .position(|t| t.id == current.id)
            .unwrap_or(0)
    } else {
        0
    };

    let has_more_tasks = total_tasks > 1;

    Ok(TaskCycleResult {
        next_task: current_task,
        has_more_tasks,
        cycle_position,
        total_tasks,
    })
}
