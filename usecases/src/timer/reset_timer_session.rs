use domain::timer::TimerService;
use domain::{EventPublisher, Result, TaskId, TaskRepository, timer::Reset};
use std::sync::Arc;

/// Reset the current timer session phase
///
/// This use case resets the current timer phase back to its full duration
/// while preserving the current phase and task context. It coordinates
/// between the timer service and task repository to provide proper context.
///
/// ## Business Rules
///
/// - Resets only the current phase, not the entire session
/// - Preserves the active task and phase context
/// - Can be called in any timer state
///
/// ## Dependencies
///
/// - TimerService: For timer operations (domain abstraction)
/// - TaskRepository: For active task context
/// - EventPublisher: For domain event publishing (business orchestration)
pub async fn reset_timer_session(
    timer_service: Arc<dyn TimerService + Send + Sync>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
) -> Result<()> {
    let current_state = timer_service.get_state().await?;

    let task = if let Some(entity_id_str) = current_state.active_entity_id() {
        if let Ok(task_id) = TaskId::from_string(&entity_id_str) {
            task_repo.get_by_id(task_id).await?
        } else {
            None
        }
    } else {
        None
    };

    // Reset the current phase with task context
    timer_service.reset_current_phase(task.as_ref()).await?;

    // Business logic: Publish Reset event after successful reset
    let updated_state = timer_service.get_state().await?;
    let timer_reset_event = Reset::new(
        updated_state.active_entity_id(),
        updated_state.phase(),
        1, // version
    );
    event_publisher.publish(Box::new(timer_reset_event));

    Ok(())
}
