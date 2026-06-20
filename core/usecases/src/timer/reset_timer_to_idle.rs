use domain::{EventPublisher, Result, TaskId, TaskRepository, TimerRepository};
use std::sync::Arc;

/// Reset the timer to the idle state
///
/// This use case stops the timer entirely and returns it to the idle state,
/// regardless of its current phase. Unlike `reset_timer_phase`, which only
/// resets the countdown within the current phase, this use case fully stops
/// the timer.
///
/// ## Business Rules
///
/// - Task must exist to provide timer configuration
/// - Transitions the timer to the idle state
/// - Does not modify task phases or task state
///
/// ## Dependencies
///
/// - TaskRepository: For retrieving task's timer configuration
/// - TimerRepository: For timer operations
/// - EventPublisher: For domain event publishing
pub async fn reset_timer_to_idle(
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

    // Reset the timer to idle with task's configuration
    let events = timer.reset(&task.config().timer)?;
    timer_repo.save(&timer).await?;

    // Publish events
    for event in events {
        event_publisher.publish(event);
    }

    Ok(())
}
