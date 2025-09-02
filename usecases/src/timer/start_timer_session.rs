use domain::{
    Error, EventPublisher, Result, TaskId, TaskRepository, 
    TimerRepository,
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
/// - TimerRepository: For timer operations
/// - EventPublisher: For domain event publishing
pub async fn start_timer_session(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: StartTimerSessionCmd,
) -> Result<()> {
    // Get task_id - either from command or use default task
    let task_id_str = cmd.task_id.ok_or_else(|| {
        Error::InvalidStateTransition {
            from: "no_task_specified".to_string(),
            to: "start_session".to_string(),
        }
    })?;

    let task_id = TaskId::from_string(&task_id_str).map_err(|_| {
        Error::TaskNotFound {
            id: task_id_str.clone(),
        }
    })?;

    // Get the task
    let task = task_repo
        .get_by_id(task_id)
        .await?
        .ok_or(Error::TaskNotFound { id: task_id_str })?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    // Get the single timer instance
    let mut timer = timer_repo.get().await?;

    // Check if timer is already running
    if timer.is_running() {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "Start".to_string(),
        });
    }

    // Set the active task on the timer
    timer.set_active_task(task_id);

    // Start the timer with the task's configuration
    let events = timer.start(&task.config.timer)?;
    timer_repo.save(&timer).await?;

    // Publish events
    for event in events {
        event_publisher.publish(event);
    }

    Ok(())
}
