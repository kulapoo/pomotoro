use domain::{Result, TaskId, TaskRepository, TimerRepository};
use std::sync::Arc;
use super::get_next_task_for_switch;

/// Get the next task to switch to (simplified version)
pub async fn switch_to_next_task(
    current_task_id: Option<TaskId>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
) -> Result<Option<String>> {
    // Check timer state
    let timer = timer_repo.get().await?;

    // Get all tasks
    let tasks = task_repo.get_all().await?;

    // Use pure function to determine next task
    let next_task = get_next_task_for_switch(
        &tasks,
        current_task_id.as_ref(),
        timer.is_running(),
    )?;

    Ok(next_task.map(|t| t.id.to_string()))
}