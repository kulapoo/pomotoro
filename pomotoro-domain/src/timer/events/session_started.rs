use serde::{Deserialize, Serialize};
use crate::{TaskId, Phase, DomainEvent};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionStarted {
    pub active_task_id: Option<TaskId>,
    pub phase: Phase,
    pub duration_seconds: u32,
    pub session_count: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl SessionStarted {
    pub fn new(
        active_task_id: Option<TaskId>,
        phase: Phase,
        duration_seconds: u32,
        session_count: u32,
        version: u64,
    ) -> Self {
        Self {
            active_task_id,
            phase,
            duration_seconds,
            session_count,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for SessionStarted {
    fn event_type(&self) -> &'static str {
        "SessionStarted"
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
}