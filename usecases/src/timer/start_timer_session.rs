use domain::{
    Error, EventPublisher, Result, TaskId, TaskRepository, TimerRepository,
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
/// - TimerRepository: For timer persistence
/// - EventPublisher: For domain event publishing
pub async fn start_timer_session(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
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

    // Load the timer aggregate
    let mut timer = timer_repo.get().await?;
    
    // Check if timer is already running
    if timer.is_running() {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "Start".to_string(),
        });
    }

    // Set the active task
    timer.set_active_task(task_id);
    
    // Execute domain logic: start the timer
    let events = timer.start(&task.config.timer)?;
    
    // Save the timer state
    timer_repo.save(&timer).await?;
    
    // Publish domain events
    for event in events {
        event_publisher.publish(event);
    }
    
    Ok(())
}
