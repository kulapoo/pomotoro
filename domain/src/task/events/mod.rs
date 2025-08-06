pub mod task_created;
pub mod task_session_completed;
pub mod task_completed;
pub mod task_status_changed;
pub mod task_updated;
pub mod session_transition_completed;
pub mod task_switch_workflow_completed;
pub mod automatic_task_cycling_completed;
pub mod task_cycling_exhausted;

pub use task_created::TaskCreated;
pub use task_session_completed::TaskSessionCompleted;
pub use task_completed::TaskCompleted;
pub use task_status_changed::TaskStatusChanged;
pub use task_updated::TaskUpdated;
pub use session_transition_completed::SessionTransitionCompleted;
pub use task_switch_workflow_completed::TaskSwitchWorkflowCompleted;
pub use automatic_task_cycling_completed::AutomaticTaskCyclingCompleted;
pub use task_cycling_exhausted::TaskCyclingExhausted;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{TaskId, TaskConfig, AudioConfig};
    use crate::DomainEvent;
    use std::time::Duration;

    #[test]
    fn should_have_correct_event_types() {
        let task_id = TaskId::new();
        let task_created = TaskCreated::new(
            task_id.clone(),
            "Test".to_string(),
            None,
            4,
            vec![],
            TaskConfig::default(),
            AudioConfig::default(),
            1,
        );
        
        assert_eq!(task_created.event_type(), "TaskCreated");
        assert_eq!(task_created.aggregate_id(), task_id.to_string());
        assert_eq!(task_created.version(), 1);
    }

    #[test]
    fn should_serialize_task_created_event() {
        let event = TaskCreated::new(
            TaskId::new(),
            "Test Task".to_string(),
            Some("A test task".to_string()),
            4,
            vec!["work".to_string()],
            TaskConfig {
                work_duration: Duration::from_secs(1500),
                short_break_duration: Duration::from_secs(300),
                long_break_duration: Duration::from_secs(900),
                sessions_until_long_break: 4,
                enable_screen_blocking: false,
            },
            AudioConfig::default(),
            1,
        );

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: TaskCreated = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn should_serialize_task_session_completed_event() {
        let event = TaskSessionCompleted::new(
            TaskId::new(),
            2,
            4,
            false,
            2,
        );

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: TaskSessionCompleted = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }
}