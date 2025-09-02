use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::TimerId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkSessionStarted {
    pub timer_id: TimerId,
    pub duration_seconds: u32,
    pub session_count: u32,
    pub task_session_count: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl WorkSessionStarted {
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
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for WorkSessionStarted {
    fn event_type(&self) -> &'static str {
        "WorkSessionStarted"
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
