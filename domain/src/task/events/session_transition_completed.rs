use serde::{Deserialize, Serialize};
use crate::TaskId;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionTransitionCompleted {
    pub task_id: TaskId,
    pub from_phase: String,
    pub to_phase: String,
    pub session_count: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl SessionTransitionCompleted {
    pub fn new(
        task_id: TaskId,
        from_phase: String,
        to_phase: String,
        session_count: u32,
        version: u64,
    ) -> Self {
        Self {
            task_id,
            from_phase,
            to_phase,
            session_count,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for SessionTransitionCompleted {
    fn event_type(&self) -> &'static str {
        "SessionTransitionCompleted"
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
