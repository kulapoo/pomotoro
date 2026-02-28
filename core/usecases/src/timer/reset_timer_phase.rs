use domain::{EventPublisher, Result, TaskId, TaskRepository, TimerRepository};
use std::sync::Arc;

/// Reset the timer countdown to its initial state
///
/// This use case resets only the timer countdown back to its initial idle state,
/// without affecting task phases. For resetting task phases, use the
/// `reset_task` use case instead.
///
/// ## Business Rules
///
/// - Task must exist to provide timer configuration
/// - Resets the timer to idle state
/// - Does not modify task phases or task state
///
/// ## Dependencies
///
/// - TaskRepository: For retrieving task's timer configuration
/// - TimerRepository: For timer operations
/// - EventPublisher: For domain event publishing
pub async fn reset_timer_phase(
    task_id: TaskId,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
) -> Result<()> {
    // Get the task only for its timer configuration
    let task = task_repo.get_by_id(task_id).await?.ok_or(
        domain::Error::TaskNotFound {
            id: task_id.to_string(),
        },
    )?;

    // Get the single timer instance
    let mut timer = timer_repo.get().await?;

    // Reset the timer with task's configuration
    let events = timer.reset_phase(&task.config.timer)?;
    timer_repo.save(&timer).await?;

    // Publish events
    for event in events {
        event_publisher.publish(event);
    }

    Ok(())
}
