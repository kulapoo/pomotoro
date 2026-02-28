use domain::{
    EventPublisher, Result, TaskId, TaskRepository, TimerRepository,
    TimerStatus,
};
use std::sync::Arc;

/// Resume a paused timer phase
///
/// Resumes a paused timer using the active task's configuration.
pub async fn resume_timer_phase(
    task_id: TaskId,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
) -> Result<TimerStatus> {
    // Load the timer aggregate
    let mut timer = timer_repo.get().await?;

    // Verify the timer is for the expected task
    if timer.task_id() != task_id {
        return Err(domain::Error::InvalidStateTransition {
            from: "different_task".to_string(),
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
