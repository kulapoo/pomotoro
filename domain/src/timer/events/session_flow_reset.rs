use crate::TimerId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionFlowReset {
    pub timer_id: TimerId,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl SessionFlowReset {
    pub fn new(timer_id: TimerId, version: u64) -> Self {
        Self {
            timer_id,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for SessionFlowReset {
    fn event_type(&self) -> &'static str {
        "SessionFlowReset"
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
