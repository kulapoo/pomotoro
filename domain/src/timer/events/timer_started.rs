use crate::timer::Phase;
use crate::{TaskId, TimerId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Started {
    pub timer_id: TimerId,
    pub phase: Phase,
    pub duration_seconds: u32,
    pub active_entity_id: Option<TaskId>,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl Started {
    pub fn new(
        timer_id: TimerId,
        phase: Phase,
        duration_seconds: u32,
        version: u64,
    ) -> Self {
        Self {
            timer_id,
            phase,
            duration_seconds,
            active_entity_id: None,
            version,
            occurred_at: Utc::now(),
        }
    }

    pub fn with_active_entity(mut self, entity_id: Option<TaskId>) -> Self {
        self.active_entity_id = entity_id;
        self
    }
}

impl crate::Event for Started {
    fn event_type(&self) -> &'static str {
        "Started"
    }

    fn aggregate_id(&self) -> String {
        self.timer_id.to_string()
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
