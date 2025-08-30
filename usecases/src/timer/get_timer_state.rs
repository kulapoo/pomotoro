use domain::timer::TimerService;
use domain::{Result, TimerState};
use std::sync::Arc;

/// Get the current timer state
///
/// This use case retrieves the current timer state from the timer service.
/// It provides a clean abstraction for controllers to access timer information
/// without directly depending on infrastructure concerns.
///
/// ## Business Rules
///
/// - Always loads the latest state from persistence
/// - Returns complete timer state information
///
/// ## Dependencies
///
/// - TimerService: For timer state access (domain abstraction)
pub async fn get_timer_state(
    timer_service: Arc<dyn TimerService + Send + Sync>,
) -> Result<TimerState> {
    timer_service.load_state().await?;

    timer_service.get_state().await
}
