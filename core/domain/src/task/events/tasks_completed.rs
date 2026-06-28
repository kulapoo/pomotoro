use crate::task::id::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Published when the last incomplete task is completed, i.e. the task
/// list is now fully done. Detected after every `TaskCompleted` emission.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TasksCompleted {
    pub completed_task_ids: Vec<Id>,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TasksCompleted {
    pub fn new(completed_task_ids: Vec<Id>) -> Self {
        Self {
            completed_task_ids,
            version: 1,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for TasksCompleted {
    fn event_type(&self) -> &'static str {
        "TasksCompleted"
    }

    fn aggregate_id(&self) -> String {
        if self.completed_task_ids.is_empty() {
            "tasks".to_string()
        } else {
            self.completed_task_ids
                .iter()
                .map(|id| id.short())
                .collect::<Vec<_>>()
                .join(",")
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Event;
    use crate::task::id::Id;

    #[test]
    fn event_type_is_tasks_completed() {
        let event = TasksCompleted::new(vec![Id::new()]);
        assert_eq!(event.event_type(), "TasksCompleted");
    }

    #[test]
    fn aggregate_id_joins_short_ids() {
        let a =
            Id::from_string("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let event = TasksCompleted::new(vec![a]);
        assert_eq!(event.aggregate_id(), "550e840");
    }

    #[test]
    fn aggregate_id_is_sentinel_when_empty() {
        let event = TasksCompleted::new(vec![]);
        assert_eq!(event.aggregate_id(), "tasks");
    }
}
