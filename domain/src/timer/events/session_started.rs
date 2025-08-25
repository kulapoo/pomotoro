use crate::timer::Phase;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionStarted {
    pub active_entity_id: Option<String>,
    pub phase: Phase,
    pub duration_seconds: u32,
    pub session_count: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl SessionStarted {
    pub fn new(
        active_entity_id: Option<String>,
        phase: Phase,
        duration_seconds: u32,
        session_count: u32,
        version: u64,
    ) -> Self {
        Self {
            active_entity_id,
            phase,
            duration_seconds,
            session_count,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for SessionStarted {
    fn event_type(&self) -> &'static str {
        "SessionStarted"
    }

    fn aggregate_id(&self) -> String {
        self.active_entity_id
            .clone()
            .unwrap_or_else(|| "timer".to_string())
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
