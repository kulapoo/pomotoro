use serde::{Deserialize, Serialize};
use crate::{TaskId, DomainEvent};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskStatusChanged {
    pub task_id: TaskId,
    pub old_status: String,
    pub new_status: String,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TaskStatusChanged {
    pub fn new(task_id: TaskId, old_status: String, new_status: String, version: u64) -> Self {
        Self {
            task_id,
            old_status,
            new_status,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for TaskStatusChanged {
    fn event_type(&self) -> &'static str {
        "TaskStatusChanged"
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

    fn clone_box(&self) -> Box<dyn DomainEvent> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}