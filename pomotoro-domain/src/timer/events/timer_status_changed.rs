use serde::{Deserialize, Serialize};
use crate::{TaskId, TimerStatus, Phase, DomainEvent};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimerStatusChanged {
    pub active_task_id: Option<TaskId>,
    pub old_status: TimerStatus,
    pub new_status: TimerStatus,
    pub phase: Phase,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TimerStatusChanged {
    pub fn new(
        active_task_id: Option<TaskId>,
        old_status: TimerStatus,
        new_status: TimerStatus,
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

impl DomainEvent for TimerStatusChanged {
    fn event_type(&self) -> &'static str {
        "TimerStatusChanged"
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