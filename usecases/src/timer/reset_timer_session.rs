use domain::{EventPublisher, Result, TaskId, TaskRepository, TimerRepository};
use std::sync::Arc;

/// Reset a timer session for a specific task
///
/// This use case resets the timer for a specific task back to its initial state.
///
/// ## Business Rules
///
/// - Task must exist
/// - Resets the timer to idle state
///
/// ## Dependencies
///
/// - TaskRepository: For task validation and retrieval
/// - TimerRepository: For timer operations
/// - EventPublisher: For domain event publishing
pub async fn reset_timer_session(
    task_id: TaskId,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
) -> Result<()> {
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

    // Reset the timer
    let events = timer.reset()?;
    timer_repo.save(timer).await?;

    // Publish events
    for event in events {
        event_publisher.publish(event);
    }

    Ok(())
}
