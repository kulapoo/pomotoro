use domain::{
    EventPublisher, Result, TaskId, TaskRepository, TimerRepository,
    TimerStatus,
};
use std::sync::Arc;

/// Pause a timer session
///
/// Pauses the currently running timer and gets configuration from
/// the active task.
///
/// ## Business Rules
///
/// - Timer must have an active task
/// - Can only pause when timer is running
/// - Returns the new status after the operation
///
/// ## Dependencies
///
/// - TaskRepository: For retrieving task configuration
/// - TimerRepository: For timer persistence
/// - EventPublisher: For domain event publishing
pub async fn pause_timer_session(
    task_id: TaskId,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
) -> Result<TimerStatus> {
    // Load the timer aggregate
    let mut timer = timer_repo.get().await?;

    // Verify the timer has the expected active task
    if timer.active_task_id() != Some(task_id) {
        return Err(domain::Error::InvalidStateTransition {
            from: "different_task_active".to_string(),
            to: "pause".to_string(),
        });
    }

    // Get the task for its configuration
    let task = task_repo.get_by_id(task_id).await?.ok_or(
        domain::Error::TaskNotFound {
            id: task_id.to_string(),
        },
    )?;

    // Execute domain logic: pause the timer
    let events = timer.pause(&task.config.timer)?;
    // Save the timer state
    timer_repo.save(&timer).await?;

    // Publish domain events
    for event in events {
        event_publisher.publish(event);
    }

    Ok(timer.status())
}

/// Resume a paused timer session
///
/// Resumes a paused timer using the active task's configuration.
pub async fn resume_timer_session(
    task_id: TaskId,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
) -> Result<TimerStatus> {
    // Load the timer aggregate
    let mut timer = timer_repo.get().await?;

    // Verify the timer has the expected active task
    if timer.active_task_id() != Some(task_id) {
        return Err(domain::Error::InvalidStateTransition {
            from: "different_task_active".to_string(),
            to: "resume".to_string(),
        });
    }

    // Get the task for its configuration
    let task = task_repo.get_by_id(task_id).await?.ok_or(
        domain::Error::TaskNotFound {
            id: task_id.to_string(),
        },
    )?;

    // Execute domain logic: resume the timer
    let events = timer.resume(&task.config.timer)?;

    // Save the timer state
    timer_repo.save(&timer).await?;

    // Publish domain events
    for event in events {
        event_publisher.publish(event);
    }

    Ok(timer.status())
}
