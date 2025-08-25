use crate::task::id::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Completed {
    pub task_id: Id,
    pub total_sessions: u8,
    pub completed_at: DateTime<Utc>,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl Completed {
    pub fn new(task_id: Id, total_sessions: u8, version: u64) -> Self {
        let now = Utc::now();
        Self {
            task_id,
            total_sessions,
            completed_at: now,
            version,
            occurred_at: now,
        }
    }
}

impl crate::Event for Completed {
    fn event_type(&self) -> &'static str {
        "TaskCompleted"
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
