use domain::{
    EventPublisher, Phase, Result, TaskId, TaskRepository, TimerRepository,
};
use std::sync::Arc;

/// Skip to the next phase in the pomodoro cycle
///
/// This use case immediately transitions the timer to the next phase
/// (work -> break -> work) and handles any necessary work session completion
/// events. It coordinates between the timer service and task repository.
///
/// ## Business Rules
///
/// - Transitions immediately to next phase regardless of remaining time
/// - May trigger work session completion events if skipping work phase
/// - Follows standard pomodoro cycle progression
///
/// ## Dependencies
///
/// - TimerService: For timer operations (domain abstraction)
/// - TaskRepository: For active task context
/// - EventPublisher: For domain event publishing (business orchestration)
///
/// ## Returns
///
/// - Tuple of (old_phase, new_phase) indicating the transition
pub async fn skip_timer_phase(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    task_id: TaskId,
) -> Result<(Phase, Phase)> {
    // Get the task for its configuration
    let task = task_repo.get_by_id(task_id).await?.ok_or(
        domain::Error::TaskNotFound {
            id: task_id.as_str(),
        },
    )?;

    let mut timer = timer_repo.get().await?;

    let old_phase = timer.get_current_phase();

    let events = timer.skip_phase(&task.config.timer)?;
    timer_repo.save(&timer).await?;

    let new_phase = timer.get_current_phase();

    for event in events {
        event_publisher.publish(event);
    }

    Ok((old_phase, new_phase))
}
