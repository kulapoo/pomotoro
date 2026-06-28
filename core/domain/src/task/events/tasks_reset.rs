use crate::task::id::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Batch event published when a user resets multiple tasks at once.
///
/// Replaces the per-task `TaskReset` spam emitted by the batch
/// `reset_tasks` usecase so the UI can show a single toast instead of N.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TasksReset {
    pub task_ids: Vec<Id>,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TasksReset {
    pub fn new(task_ids: Vec<Id>) -> Self {
        Self {
            task_ids,
            version: 1,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for TasksReset {
    fn event_type(&self) -> &'static str {
        "TasksReset"
    }

    fn aggregate_id(&self) -> String {
        if self.task_ids.is_empty() {
            "tasks".to_string()
        } else {
            self.task_ids
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
    fn event_type_is_tasks_reset() {
        let event = TasksReset::new(vec![Id::new(), Id::new()]);
        assert_eq!(event.event_type(), "TasksReset");
    }

    #[test]
    fn aggregate_id_joins_short_ids() {
        let a =
            Id::from_string("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let b =
            Id::from_string("12345678-e29b-41d4-a716-446655440000").unwrap();
        let event = TasksReset::new(vec![a, b]);
        assert_eq!(event.aggregate_id(), "550e840,1234567");
    }

    #[test]
    fn aggregate_id_is_sentinel_when_empty() {
        let event = TasksReset::new(vec![]);
        assert_eq!(event.aggregate_id(), "tasks");
    }
}
