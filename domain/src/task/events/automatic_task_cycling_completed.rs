use serde::{Deserialize, Serialize};
use crate::{TaskId, DomainEvent};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomaticTaskCyclingCompleted {
    pub current_task_id: TaskId,
    pub next_task_id: Option<TaskId>,
    pub cycle_count: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl AutomaticTaskCyclingCompleted {
    pub fn new(
        current_task_id: TaskId,
        next_task_id: Option<TaskId>,
        cycle_count: u32,
        version: u64,
    ) -> Self {
        Self {
            current_task_id,
            next_task_id,
            cycle_count,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for AutomaticTaskCyclingCompleted {
    fn event_type(&self) -> &'static str {
        "AutomaticTaskCyclingCompleted"
    }

    fn aggregate_id(&self) -> String {
        self.current_task_id.to_string()
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}