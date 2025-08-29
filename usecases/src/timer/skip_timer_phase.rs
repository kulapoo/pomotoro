use domain::timer::TimerService;
use domain::{
    EventPublisher, Phase, Result, TaskId, TaskRepository, WorkSessionCompleted,
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
    timer_service: &Arc<dyn TimerService + Send + Sync>,
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<(Phase, Phase)> {
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

    // Store initial state for business logic decisions
    let old_phase = current_state.phase();

    // Skip to the next phase with task context
    let (_, new_phase) =
        timer_service.skip_to_next_phase(task.as_ref()).await?;

    // Business logic: Publish WorkSessionCompleted event if we completed a work session
    if old_phase == Phase::Work
        && (new_phase == Phase::ShortBreak || new_phase == Phase::LongBreak)
    {
        if let Some(task_ref) = &task {
            let updated_state = timer_service.get_state().await?;
            let work_session_event = WorkSessionCompleted::new(
                Some(task_ref.id.to_string()),
                1500, // 25 minutes work session default duration (TODO: get from task config)
                updated_state.session_count(),
                task_ref.current_sessions as u32 + 1, // increment since we just completed
                1,                                    // version
            );
            event_publisher.publish(Box::new(work_session_event));
        }
    }

    Ok((old_phase, new_phase))
}
