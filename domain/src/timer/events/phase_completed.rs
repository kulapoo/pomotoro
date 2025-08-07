use serde::{Deserialize, Serialize};
use crate::{TaskId, Phase, DomainEvent};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PhaseCompleted {
    pub active_task_id: Option<TaskId>,
    pub completed_phase: Phase,
    pub next_phase: Phase,
    pub session_count: u32,
    pub task_session_count: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl PhaseCompleted {
    pub fn new(
        active_task_id: Option<TaskId>,
        completed_phase: Phase,
        next_phase: Phase,
        session_count: u32,
        task_session_count: u32,
        version: u64,
    ) -> Self {
        Self {
            active_task_id,
            completed_phase,
            next_phase,
            session_count,
            task_session_count,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for PhaseCompleted {
    fn event_type(&self) -> &'static str {
        "PhaseCompleted"
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

    fn clone_box(&self) -> Box<dyn DomainEvent> {
        Box::new(self.clone())
    }
}