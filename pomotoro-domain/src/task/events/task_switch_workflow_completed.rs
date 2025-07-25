use serde::{Deserialize, Serialize};
use crate::{TaskId, DomainEvent};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskSwitchWorkflowCompleted {
    pub old_task_id: Option<TaskId>,
    pub new_task_id: TaskId,
    pub workflow_result: String,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TaskSwitchWorkflowCompleted {
    pub fn new(
        old_task_id: Option<TaskId>,
        new_task_id: TaskId,
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

impl DomainEvent for TaskSwitchWorkflowCompleted {
    fn event_type(&self) -> &'static str {
        "TaskSwitchWorkflowCompleted"
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
}