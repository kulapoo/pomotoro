use crate::task::id::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionCompleted {
    pub task_id: Id,
    pub session_number: u8,
    pub total_sessions: u8,
    pub is_task_completed: bool,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl SessionCompleted {
    pub fn new(
        task_id: Id,
        session_number: u8,
        total_sessions: u8,
        is_task_completed: bool,
        version: u64,
    ) -> Self {
        Self {
            task_id,
            session_number,
            total_sessions,
            is_task_completed,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for SessionCompleted {
    fn event_type(&self) -> &'static str {
        "TaskSessionCompleted"
    }

    fn aggregate_id(&self) -> String {
        self.task_id.to_string()
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn clone_box(&self) -> Box<dyn crate::Event> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
