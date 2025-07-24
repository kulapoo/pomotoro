use serde::{Deserialize, Serialize};
use crate::{TaskId, TaskConfig, AudioConfig, DomainEvent};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskCreated {
    pub task_id: TaskId,
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub tags: Vec<String>,
    pub config: TaskConfig,
    pub audio_config: AudioConfig,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TaskCreated {
    pub fn new(
        task_id: TaskId,
        name: String,
        description: Option<String>,
        max_sessions: u8,
        tags: Vec<String>,
        config: TaskConfig,
        audio_config: AudioConfig,
        version: u64,
    ) -> Self {
        Self {
            task_id,
            name,
            description,
            max_sessions,
            tags,
            config,
            audio_config,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for TaskCreated {
    fn event_type(&self) -> &'static str {
        "TaskCreated"
    }

    fn aggregate_id(&self) -> String {
        self.task_id.to_string()
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskSessionCompleted {
    pub task_id: TaskId,
    pub session_number: u8,
    pub total_sessions: u8,
    pub is_task_completed: bool,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TaskSessionCompleted {
    pub fn new(
        task_id: TaskId,
        session_number: u8,
        total_sessions: u8,
        is_task_completed: bool,
        version: u64,
    ) -> Self {
        Self {
            task_id,
            session_number,
            total_sessions,
            is_task_completed,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for TaskSessionCompleted {
    fn event_type(&self) -> &'static str {
        "TaskSessionCompleted"
    }

    fn aggregate_id(&self) -> String {
        self.task_id.to_string()
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskCompleted {
    pub task_id: TaskId,
    pub total_sessions: u8,
    pub completed_at: DateTime<Utc>,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TaskCompleted {
    pub fn new(task_id: TaskId, total_sessions: u8, version: u64) -> Self {
        let now = Utc::now();
        Self {
            task_id,
            total_sessions,
            completed_at: now,
            version,
            occurred_at: now,
        }
    }
}

impl DomainEvent for TaskCompleted {
    fn event_type(&self) -> &'static str {
        "TaskCompleted"
    }

    fn aggregate_id(&self) -> String {
        self.task_id.to_string()
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskStatusChanged {
    pub task_id: TaskId,
    pub old_status: String,
    pub new_status: String,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TaskStatusChanged {
    pub fn new(task_id: TaskId, old_status: String, new_status: String, version: u64) -> Self {
        Self {
            task_id,
            old_status,
            new_status,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for TaskStatusChanged {
    fn event_type(&self) -> &'static str {
        "TaskStatusChanged"
    }

    fn aggregate_id(&self) -> String {
        self.task_id.to_string()
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskUpdated {
    pub task_id: TaskId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_sessions: Option<u8>,
    pub tags: Option<Vec<String>>,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TaskUpdated {
    pub fn new(
        task_id: TaskId,
        name: Option<String>,
        description: Option<String>,
        max_sessions: Option<u8>,
        tags: Option<Vec<String>>,
        version: u64,
    ) -> Self {
        Self {
            task_id,
            name,
            description,
            max_sessions,
            tags,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl DomainEvent for TaskUpdated {
    fn event_type(&self) -> &'static str {
        "TaskUpdated"
    }

    fn aggregate_id(&self) -> String {
        self.task_id.to_string()
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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