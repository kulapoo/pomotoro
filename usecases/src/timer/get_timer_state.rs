use domain::{Result, TimerState, TimerRepository};
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
/// - Returns complete timer state information
///
/// ## Dependencies
///
/// - TimerRepository: For timer persistence
pub async fn get_timer_state(
    timer_repo: Arc<dyn TimerRepository + Send + Sync>,
) -> Result<TimerState> {
    // Load the timer aggregate
    let timer = timer_repo.get().await?;
    
    // Return the timer's state
    Ok(timer.state().clone())
}
