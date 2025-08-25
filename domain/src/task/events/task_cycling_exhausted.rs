use crate::task::id::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CyclingExhausted {
    pub current_task_id: Id,
    pub attempted_cycles: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl CyclingExhausted {
    pub fn new(
        current_task_id: Id,
        attempted_cycles: u32,
        version: u64,
    ) -> Self {
        Self {
            current_task_id,
            attempted_cycles,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for CyclingExhausted {
    fn event_type(&self) -> &'static str {
        "CyclingExhausted"
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
