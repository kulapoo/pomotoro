use crate::TaskId;
use crate::timer::Phase;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseCompleted {
    pub task_id: TaskId,
    pub completed_phase: Phase,
    pub next_phase: Phase,
    pub session_count: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl PhaseCompleted {
    pub fn new(
        task_id: TaskId,
        completed_phase: Phase,
        next_phase: Phase,
        session_count: u32,
        version: u64,
    ) -> Self {
        Self {
            task_id,
            completed_phase,
            next_phase,
            session_count,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for PhaseCompleted {
    fn event_type(&self) -> &'static str {
        "PhaseCompleted"
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
