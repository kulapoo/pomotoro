use serde::{Deserialize, Serialize};
use crate::task::id::Id as TaskId;
use crate::timer::{status::Status, Phase};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusChanged {
    pub active_task_id: Option<TaskId>,
    pub old_status: Status,
    pub new_status: Status,
    pub phase: Phase,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl StatusChanged {
    pub fn new(
        active_task_id: Option<TaskId>,
        old_status: Status,
        new_status: Status,
        phase: Phase,
        version: u64,
    ) -> Self {
        Self {
            active_task_id,
            old_status,
            new_status,
            phase,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for StatusChanged {
    fn event_type(&self) -> &'static str {
        "StatusChanged"
    }

    fn aggregate_id(&self) -> String {
        self.active_task_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| "timer".to_string())
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
