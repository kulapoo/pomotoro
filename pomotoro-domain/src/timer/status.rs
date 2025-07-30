use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TimerStatus {
    Stopped,
    Running,
    Paused,
}

impl TimerStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, TimerStatus::Running)
    }

    pub fn can_transition_to(&self, new_status: &TimerStatus) -> bool {
        matches!((self, new_status), (TimerStatus::Stopped, TimerStatus::Running) | (TimerStatus::Running, TimerStatus::Paused) | (TimerStatus::Paused, TimerStatus::Running) | (_, TimerStatus::Stopped))
    }
}