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
/// In the new architecture, each task has its own timer,
/// so we start the timer associated with the specific task.
///
/// ## Business Rules
///
/// - Task must exist and not be completed
/// - Task's timer must not already be running
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

    // Get the task's timer
    let mut timer = timer_repo
        .get_by_id(task.get_timer_id())
        .await?
        .ok_or_else(|| Error::RepositoryError {
            message: format!("Timer not found for task: {}", task.name),
        })?;

    // Check if timer is already running
    if timer.is_running() {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "Start".to_string(),
        });
    }

    // Start the timer
    let events = timer.start()?;
    timer_repo.save(timer.clone()).await?;

    // Publish events
    for event in events {
        event_publisher.publish(event);
    }

    Ok(())
}
