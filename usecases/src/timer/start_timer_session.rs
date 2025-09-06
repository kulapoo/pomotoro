use domain::{
    Error, EventPublisher, Result, TaskId, TaskRepository, TimerRepository,
    timer::TimerService,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StartTimerSessionCmd {
    pub task_id: Option<String>,
}

/// Start a timer session for a specific task
///
/// Sets the active task on the timer and starts it using
/// the task's configuration.
///
/// ## Business Rules
///
/// - Task must exist and not be completed
/// - Timer must not already be running
///
/// ## Dependencies
///
/// - TaskRepository: For task validation and retrieval
/// - TimerService: Timer service helpers
/// - EventPublisher: For domain event publishing
pub async fn start_timer_session(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_service: Arc<dyn TimerService + Send + Sync>,
    cmd: StartTimerSessionCmd,
) -> Result<()> {
    // Closure that preserves error context
    let to_invalid_task_data =
        |msg: String| domain::Error::InvalidTaskParams { message: msg };

    let task_id_str = cmd.task_id.ok_or_else(|| {
        to_invalid_task_data("Task ID is required".to_string())
    })?;

    let task_id = TaskId::from_string(&task_id_str).map_err(|e| {
        to_invalid_task_data(format!("Invalid task ID format: {}", e))
    })?;

    let task = task_repo
        .get_by_id(task_id)
        .await?
        .ok_or(Error::TaskNotFound { id: task_id_str })?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    // Get the single timer instance
    let timer = timer_service.get_timer().await?;
    // // Check if timer is already running
    if timer.is_running() {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "Start".to_string(),
        });
    }

    timer_service.switch_task(task_id, Some(&task)).await?;

    timer_service.start_timer(Some(&task)).await?;
    Ok(())
}
