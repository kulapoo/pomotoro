use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::TimerId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkSessionCompleted {
    pub timer_id: TimerId,
    pub duration_seconds: u32,
    pub session_count: u32,
    pub task_session_count: u32,
    pub active_entity_id: Option<String>,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl WorkSessionCompleted {
    pub fn new(
        timer_id: TimerId,
        duration_seconds: u32,
        session_count: u32,
        task_session_count: u32,
        version: u64,
    ) -> Self {
        Self {
            timer_id,
            duration_seconds,
            session_count,
            task_session_count,
            active_entity_id: None,
            version,
            occurred_at: Utc::now(),
        }
    }
    
    pub fn with_active_entity(mut self, entity_id: Option<String>) -> Self {
        self.active_entity_id = entity_id;
        self
    }
}

impl crate::Event for WorkSessionCompleted {
    fn event_type(&self) -> &'static str {
        "WorkSessionCompleted"
    }

    fn aggregate_id(&self) -> String {
        self.timer_id.to_string()
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
