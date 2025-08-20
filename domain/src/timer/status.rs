use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Stopped,
    Running,
    Paused,
}

impl Status {
    pub fn is_active(&self) -> bool {
        matches!(self, Status::Running)
    }

    pub fn can_transition_to(&self, new_status: &Status) -> bool {
        matches!((self, new_status), (Status::Stopped, Status::Running) | (Status::Running, Status::Paused) | (Status::Paused, Status::Running) | (_, Status::Stopped))
    }
}