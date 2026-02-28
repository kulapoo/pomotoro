use crate::TaskId;
use crate::timer::Phase;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseSkipped {
    pub task_id: TaskId,
    pub skipped_phase: Phase,
    pub next_phase: Phase,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl PhaseSkipped {
    pub fn new(
        task_id: TaskId,
        skipped_phase: Phase,
        next_phase: Phase,
        version: u64,
    ) -> Self {
        Self {
            task_id,
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
