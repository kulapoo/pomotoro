use serde::{Deserialize, Serialize};

use crate::timer::Phase;
use crate::TimerId;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Paused {
    pub timer_id: TimerId,
    pub phase: Phase,
    pub remaining_seconds: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl Paused {
    pub fn new(
        timer_id: TimerId,
        phase: Phase,
        remaining_seconds: u32,
        version: u64,
    ) -> Self {
        Self {
            timer_id,
            phase,
            remaining_seconds,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for Paused {
    fn event_type(&self) -> &'static str {
        "Paused"
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
