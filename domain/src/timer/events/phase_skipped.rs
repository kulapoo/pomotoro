use crate::TimerId;
use crate::timer::Phase;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseSkipped {
    pub timer_id: TimerId,
    pub skipped_phase: Phase,
    pub next_phase: Phase,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl PhaseSkipped {
    pub fn new(
        timer_id: TimerId,
        skipped_phase: Phase,
        next_phase: Phase,
        version: u64,
    ) -> Self {
        Self {
            timer_id,
            skipped_phase,
            next_phase,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for PhaseSkipped {
    fn event_type(&self) -> &'static str {
        "PhaseSkipped"
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
