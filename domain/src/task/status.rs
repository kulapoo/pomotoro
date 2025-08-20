use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Active,
    Queued,
    Completed,
    Paused,
}

impl Status {
    pub fn is_active(&self) -> bool {
        matches!(self, Status::Active)
    }

    pub fn is_completed(&self) -> bool {
        matches!(self, Status::Completed)
    }

    pub fn can_be_started(&self) -> bool {
        matches!(self, Status::Active | Status::Queued)
    }
}