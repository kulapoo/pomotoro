use domain::timer::TimerService;
use domain::{EventPublisher, Result, TimerStatus, timer::Paused};
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
    timer_service: &Arc<dyn TimerService + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<TimerStatus> {
    let new_status = timer_service.toggle_pause().await?;

    // Business logic: Publish Paused event when timer becomes paused
    if new_status == TimerStatus::Paused {
        let state = timer_service.get_state().await?;
        let timer_paused_event = Paused::new(
            state.active_entity_id(),
            state.phase(),
            state.remaining_seconds(),
            1, // version
        );
        event_publisher.publish(Box::new(timer_paused_event));
    }

    Ok(new_status)
}

/// Resume a paused timer session
///
/// This is an alias for pause_timer_session for better semantic clarity
/// when the intent is specifically to resume a paused timer.
pub async fn resume_timer_session(
    timer_service: &Arc<dyn TimerService + Send + Sync>,
    _event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<TimerStatus> {
    timer_service.toggle_pause().await
}
