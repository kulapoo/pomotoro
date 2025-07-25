use serde::{Deserialize, Serialize};
use crate::{TaskId, Phase, DomainEvent};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseSkipped {
    pub active_task_id: Option<TaskId>,
    pub skipped_phase: Phase,
    pub next_phase: Phase,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl PhaseSkipped {
    pub fn new(
        active_task_id: Option<TaskId>,
        skipped_phase: Phase,
        next_phase: Phase,
        version: u64,
    ) -> Self {
        Self {
            active_task_id,
            skipped_phase,
            next_phase,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for PhaseSkipped {
    fn event_type(&self) -> &'static str {
        "PhaseSkipped"
    }

    fn aggregate_id(&self) -> String {
        self.active_task_id.clone()
            .map(|id| id.to_string())
            .unwrap_or_else(|| "timer".to_string())
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}