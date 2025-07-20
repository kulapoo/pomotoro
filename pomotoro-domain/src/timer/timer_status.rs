use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
        match (self, new_status) {
            (TimerStatus::Stopped, TimerStatus::Running) => true,
            (TimerStatus::Running, TimerStatus::Paused) => true,
            (TimerStatus::Paused, TimerStatus::Running) => true,
            (_, TimerStatus::Stopped) => true,
            _ => false,
        }
    }
}