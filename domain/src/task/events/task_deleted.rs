use crate::Event;
use crate::task::id::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]

pub struct TaskDeleted {
    pub task_id: Id,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TaskDeleted {
    pub fn new(task_id: Id, version: u64) -> Self {
        Self {
            task_id,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl Event for TaskDeleted {
    fn event_type(&self) -> &'static str {
        "TaskDeleted"
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

    fn clone_box(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
