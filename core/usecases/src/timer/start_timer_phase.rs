use domain::{
    Error, EventPublisher, Result, TaskId, TaskRepository, TimerRepository,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StartTimerPhaseCmd {
    pub task_id: Option<TaskId>,
}

/// Start a timer phase for a specific task
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
pub async fn start_timer_phase(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: StartTimerPhaseCmd,
) -> Result<()> {
    let to_invalid_task_data =
        |msg: String| domain::Error::InvalidTaskParams { message: msg };

    let task_id = cmd.task_id.ok_or_else(|| {
        to_invalid_task_data("Task ID is required".to_string())
    })?;

    let mut task =
        task_repo
            .get_by_id(task_id)
            .await?
            .ok_or(Error::TaskNotFound {
                id: task_id.as_str(),
            })?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    // Check if timer is already running
    let existing_timer = timer_repo.get().await?;
    if existing_timer.is_running() {
        return Err(Error::InvalidStateTransition {
            from: "Running".to_string(),
            to: "Start".to_string(),
        });
    }

    // Get the timer config before moving the task
    let timer_config = task.config().timer.clone();

    // Set the new task status to Active
    task.activate()?;

    let task_id_for_event = task.id();

    task_repo.update(task).await?;

    // Emit TaskUpdated event for new active task
    event_publisher.publish(Box::new(domain::TaskUpdated::new(
        task_id_for_event,
        None,
        None,
        None,
        None,
        1,
    )));

    // Create a new timer with the correct task_id
    let mut timer = domain::Timer::new(task_id_for_event);
    let events = timer
        .as_active_mut()
        .ok_or(domain::Error::NoActiveTask)?
        .start(&timer_config)?;

    timer_repo.save(&timer).await?;

    for event in events {
        event_publisher.publish(event);
    }

    Ok(())
}
