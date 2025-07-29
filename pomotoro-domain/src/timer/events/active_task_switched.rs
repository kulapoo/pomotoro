use serde::{Deserialize, Serialize};
use crate::{TaskId, Phase, DomainEvent};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveTaskSwitched {
    pub old_task_id: Option<TaskId>,
    pub new_task_id: Option<TaskId>,
    pub phase: Phase,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl ActiveTaskSwitched {
    pub fn new(
        old_task_id: Option<TaskId>,
        new_task_id: Option<TaskId>,
        phase: Phase,
        version: u64,
    ) -> Self {
        Self {
            old_task_id,
            new_task_id,
            phase,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for ActiveTaskSwitched {
    fn event_type(&self) -> &'static str {
        "ActiveTaskSwitched"
    }

    fn aggregate_id(&self) -> String {
        self.new_task_id
            .or(self.old_task_id)
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