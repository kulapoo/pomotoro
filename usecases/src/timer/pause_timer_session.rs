use domain::{EventPublisher, Result, TaskId, TaskRepository, TimerRepository, TimerStatus};
use std::sync::Arc;

/// Pause or resume a timer session
///
/// This use case toggles the timer state between running and paused.
/// It uses the timer service abstraction to handle the state transition
/// while maintaining proper business logic.
///
/// ## Business Rules
///
/// - Can only pause/resume when timer is in Running or Paused state
/// - Returns the new status after the operation
///
/// ## Dependencies
///
/// - TimerService: For timer state management (domain abstraction)
/// - EventPublisher: For domain event publishing (business orchestration)
pub async fn pause_timer_session(
    task_id: TaskId,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
) -> Result<TimerStatus> {
    // Get the task
    let task = task_repo
        .get_by_id(task_id)
        .await?
        .ok_or(domain::Error::TaskNotFound {
            id: task_id.to_string(),
        })?;

    // Get the task's timer
    let mut timer = timer_repo
        .get_by_id(task.get_timer_id())
        .await?
        .ok_or_else(|| domain::Error::RepositoryError {
            message: format!("Timer not found for task: {}", task_id),
        })?;

    // Pause the timer
    let events = timer.pause()?;
    timer_repo.save(timer.clone()).await?;

    // Publish events
    for event in events {
        event_publisher.publish(event);
    }

    Ok(timer.status())
}

/// Resume a paused timer session
///
/// This is an alias for pause_timer_session for better semantic clarity
/// when the intent is specifically to resume a paused timer.
pub async fn resume_timer_session(
    task_id: TaskId,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
) -> Result<TimerStatus> {
    // Get the task
    let task = task_repo
        .get_by_id(task_id)
        .await?
        .ok_or(domain::Error::TaskNotFound {
            id: task_id.to_string(),
        })?;

    // Get the task's timer
    let mut timer = timer_repo
        .get_by_id(task.get_timer_id())
        .await?
        .ok_or_else(|| domain::Error::RepositoryError {
            message: format!("Timer not found for task: {}", task_id),
        })?;

    // Resume the timer
    let events = timer.resume()?;
    timer_repo.save(timer.clone()).await?;

    // Publish events
    for event in events {
        event_publisher.publish(event);
    }

    Ok(timer.status())
}
