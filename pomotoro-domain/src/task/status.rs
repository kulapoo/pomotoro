use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Active,
    Queued,
    Completed,
    Paused,
}

impl TaskStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, TaskStatus::Active)
    }

    pub fn is_completed(&self) -> bool {
        matches!(self, TaskStatus::Completed)
    }

    pub fn can_be_started(&self) -> bool {
        matches!(self, TaskStatus::Active | TaskStatus::Queued)
    }
}