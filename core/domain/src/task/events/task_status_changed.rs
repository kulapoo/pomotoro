use crate::task::id::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusChanged {
    pub task_id: Id,
    pub old_status: String,
    pub new_status: String,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl StatusChanged {
    pub fn new(
        task_id: Id,
        old_status: String,
        new_status: String,
        version: u64,
    ) -> Self {
        Self {
            task_id,
            old_status,
            new_status,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for StatusChanged {
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

    fn clone_box(&self) -> Box<dyn crate::Event> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
