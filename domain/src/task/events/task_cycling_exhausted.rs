use serde::{Deserialize, Serialize};
use crate::TaskId;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskCyclingExhausted {
    pub current_task_id: TaskId,
    pub attempted_cycles: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TaskCyclingExhausted {
    pub fn new(current_task_id: TaskId, attempted_cycles: u32, version: u64) -> Self {
        Self {
            current_task_id,
            attempted_cycles,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for TaskCyclingExhausted {
    fn event_type(&self) -> &'static str {
        "TaskCyclingExhausted"
    }

    fn aggregate_id(&self) -> String {
        self.current_task_id.to_string()
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
