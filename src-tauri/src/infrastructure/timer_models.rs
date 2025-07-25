use serde::{Deserialize, Serialize};
use pomotoro_domain::{TimerState, Task};

/// Infrastructure-specific model that combines timer state with task information
/// for frontend consumption. This is not part of the domain layer as it's 
/// specifically for API/UI concerns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerStateWithTask {
    pub timer_state: TimerState,
    pub active_task: Option<Task>,
}

impl TimerStateWithTask {
    pub fn new(timer_state: TimerState, active_task: Option<Task>) -> Self {
        Self {
            timer_state,
            active_task,
        }
    }
}