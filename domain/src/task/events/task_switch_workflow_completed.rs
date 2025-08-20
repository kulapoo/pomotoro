use serde::{Deserialize, Serialize};
use crate::task::id::Id;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SwitchWorkflowCompleted {
    pub old_task_id: Option<Id>,
    pub new_task_id: Id,
    pub workflow_result: String,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl SwitchWorkflowCompleted {
    pub fn new(
        old_task_id: Option<Id>,
        new_task_id: Id,
        workflow_result: String,
        version: u64,
    ) -> Self {
        Self {
            old_task_id,
            new_task_id,
            workflow_result,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for SwitchWorkflowCompleted {
    fn event_type(&self) -> &'static str {
        "SwitchWorkflowCompleted"
    }

    fn aggregate_id(&self) -> String {
        self.new_task_id.to_string()
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
