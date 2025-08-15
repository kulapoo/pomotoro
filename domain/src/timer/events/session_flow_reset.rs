use serde::{Deserialize, Serialize};
use crate::{TaskId, DomainEvent};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionFlowReset {
    pub active_task_id: Option<TaskId>,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl SessionFlowReset {
    pub fn new(active_task_id: Option<TaskId>, version: u64) -> Self {
        Self {
            active_task_id,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for SessionFlowReset {
    fn event_type(&self) -> &'static str {
        "SessionFlowReset"
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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}