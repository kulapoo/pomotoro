use serde::{Deserialize, Serialize};
use crate::{TaskId, DomainEvent};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskCompleted {
    pub task_id: TaskId,
    pub total_sessions: u8,
    pub completed_at: DateTime<Utc>,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TaskCompleted {
    pub fn new(task_id: TaskId, total_sessions: u8, version: u64) -> Self {
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

impl DomainEvent for TaskCompleted {
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
}