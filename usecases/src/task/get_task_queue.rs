use domain::{
    Error, Result, Task, TaskCyclerService, TaskId, TaskRepository, TaskStatus,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TaskQueueQuery {
    pub include_completed: bool,
    pub active_task_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TaskQueueInfo {
    pub tasks: Vec<Task>,
    pub active_task_id: Option<TaskId>,
    pub current_position: Option<usize>,
    pub total_tasks: usize,
    pub active_tasks: usize,
    pub completed_tasks: usize,
}

impl TaskQueueInfo {
    pub fn active_task_id(&self) -> Option<TaskId> {
        self.active_task_id
    }
}

pub async fn get_task_queue(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
    query: TaskQueueQuery,
) -> Result<TaskQueueInfo> {
    let active_task_id = if let Some(id_str) = query.active_task_id {
        Some(
            TaskId::from_string(&id_str)
                .map_err(|_| Error::TaskNotFound { id: id_str })?,
        )
    } else {
        None
    };

    let tasks = if query.include_completed {
        task_repo.get_all().await?
    } else {
        cycling_service.get_active_task_queue().await?
    };

    let total_tasks = tasks.len();
    let active_tasks = tasks.iter().filter(|t| !t.is_completed()).count();
    let completed_tasks = tasks.iter().filter(|t| t.is_completed()).count();

    let current_position = if let Some(active_id) = &active_task_id {
        tasks.iter().position(|t| t.id == *active_id)
    } else {
        None
    };

    Ok(TaskQueueInfo {
        tasks,
        active_task_id,
        current_position,
        total_tasks,
        active_tasks,
        completed_tasks,
    })
}

pub async fn get_active_task_queue(
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
) -> Result<Vec<Task>> {
    cycling_service.get_active_task_queue().await
}

pub async fn get_task_queue_with_priorities(
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
    active_task_id: Option<String>,
) -> Result<TaskQueueInfo> {
    let active_id = if let Some(id_str) = active_task_id {
        Some(
            TaskId::from_string(&id_str)
                .map_err(|_| Error::TaskNotFound { id: id_str })?,
        )
    } else {
        None
    };

    let mut tasks = cycling_service.get_active_task_queue().await?;

    tasks.sort_by(|a, b| {
        if let Some(active_id) = &active_id {
            if a.id == *active_id {
                return std::cmp::Ordering::Less;
            }
            if b.id == *active_id {
                return std::cmp::Ordering::Greater;
            }
        }

        match (a.is_completed(), b.is_completed()) {
            (false, true) => std::cmp::Ordering::Less,
            (true, false) => std::cmp::Ordering::Greater,
            _ => {
                if !a.is_completed() && !b.is_completed() {
                    b.created_at.cmp(&a.created_at)
                } else {
                    a.created_at.cmp(&b.created_at)
                }
            }
        }
    });

    let total_tasks = tasks.len();
    let active_tasks = tasks.iter().filter(|t| !t.is_completed()).count();
    let completed_tasks = tasks.iter().filter(|t| t.is_completed()).count();

    let current_position = if let Some(active_id) = &active_id {
        tasks.iter().position(|t| t.id == *active_id)
    } else {
        None
    };

    Ok(TaskQueueInfo {
        tasks,
        active_task_id: active_id,
        current_position,
        total_tasks,
        active_tasks,
        completed_tasks,
    })
}

pub async fn get_task_queue_summary(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    cycling_service: &Arc<dyn TaskCyclerService + Send + Sync>,
) -> Result<TaskQueueSummary> {
    let all_tasks = task_repo.get_all().await?;
    let active_tasks = cycling_service.get_active_task_queue().await?;

    let total_tasks = all_tasks.len();
    let active_count = active_tasks.len();
    let completed_count = all_tasks.iter().filter(|t| t.is_completed()).count();
    let paused_count = all_tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Paused)
        .count();

    let total_sessions: u32 =
        all_tasks.iter().map(|t| t.max_sessions as u32).sum();
    let completed_sessions: u32 =
        all_tasks.iter().map(|t| t.current_sessions as u32).sum();

    Ok(TaskQueueSummary {
        total_tasks,
        active_tasks: active_count,
        completed_tasks: completed_count,
        paused_tasks: paused_count,
        total_sessions,
        completed_sessions,
        progress_percentage: if total_sessions > 0 {
            (completed_sessions as f64 / total_sessions as f64) * 100.0
        } else {
            0.0
        },
    })
}

#[derive(Debug, Clone)]
pub struct TaskQueueSummary {
    pub total_tasks: usize,
    pub active_tasks: usize,
    pub completed_tasks: usize,
    pub paused_tasks: usize,
    pub total_sessions: u32,
    pub completed_sessions: u32,
    pub progress_percentage: f64,
}

