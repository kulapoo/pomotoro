use serde::{Deserialize, Serialize};
use crate::{TaskId, TaskConfig, AudioConfig};
use super::domain_event::DomainEventData;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskCreated {
    pub task_id: TaskId,
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub tags: Vec<String>,
    pub config: TaskConfig,
    pub audio_config: AudioConfig,
}

impl DomainEventData for TaskCreated {
    fn event_type() -> &'static str {
        "TaskCreated"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskSessionCompleted {
    pub task_id: TaskId,
    pub session_number: u8,
    pub total_sessions: u8,
    pub is_task_completed: bool,
}

impl DomainEventData for TaskSessionCompleted {
    fn event_type() -> &'static str {
        "TaskSessionCompleted"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskCompleted {
    pub task_id: TaskId,
    pub total_sessions: u8,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

impl DomainEventData for TaskCompleted {
    fn event_type() -> &'static str {
        "TaskCompleted"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskStatusChanged {
    pub task_id: TaskId,
    pub old_status: String,
    pub new_status: String,
}

impl DomainEventData for TaskStatusChanged {
    fn event_type() -> &'static str {
        "TaskStatusChanged"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskUpdated {
    pub task_id: TaskId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_sessions: Option<u8>,
    pub tags: Option<Vec<String>>,
}

impl DomainEventData for TaskUpdated {
    fn event_type() -> &'static str {
        "TaskUpdated"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use std::time::Duration;

    #[test]
    fn should_have_correct_event_types() {
        assert_eq!(TaskCreated::event_type(), "TaskCreated");
        assert_eq!(TaskSessionCompleted::event_type(), "TaskSessionCompleted");
        assert_eq!(TaskCompleted::event_type(), "TaskCompleted");
        assert_eq!(TaskStatusChanged::event_type(), "TaskStatusChanged");
        assert_eq!(TaskUpdated::event_type(), "TaskUpdated");
    }

    #[test]
    fn should_serialize_task_created_event() {
        let event = TaskCreated {
            task_id: Uuid::new_v4(),
            name: "Test Task".to_string(),
            description: Some("A test task".to_string()),
            max_sessions: 4,
            tags: vec!["work".to_string()],
            config: TaskConfig {
                work_duration: Duration::from_secs(1500),
                short_break_duration: Duration::from_secs(300),
                long_break_duration: Duration::from_secs(900),
                sessions_until_long_break: 4,
                enable_screen_blocking: false,
            },
            audio_config: AudioConfig::default(),
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: TaskCreated = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn should_serialize_task_session_completed_event() {
        let event = TaskSessionCompleted {
            task_id: Uuid::new_v4(),
            session_number: 2,
            total_sessions: 4,
            is_task_completed: false,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: TaskSessionCompleted = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }
}