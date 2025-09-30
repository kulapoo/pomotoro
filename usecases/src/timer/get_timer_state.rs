use domain::{Result, TimerInfo, TimerRepository};
use std::sync::Arc;

/// Get the current timer state
///
/// This use case retrieves the current timer state from the timer repository.
/// It provides a clean abstraction for controllers to access timer information
/// without directly depending on infrastructure concerns.
///
/// ## Business Rules
///
/// - Always loads the latest state from persistence
/// - Returns complete timer state information including active task
///
/// ## Dependencies
///
/// - TimerRepository: For timer persistence
pub async fn get_timer_state(
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
) -> Result<TimerInfo> {
    // Load the timer aggregate
    let timer = timer_repo.get().await?;

    // Return the complete timer information
    Ok(TimerInfo::from_timer(&timer))
}
