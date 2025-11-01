use crate::Event;
use crate::timer::Phase;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;

/// Event emitted when a timer countdown naturally expires (reaches 0:00)
/// This event indicates that the timer phase completed naturally through
/// countdown expiration, not through manual skipping or stopping.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CountdownExpired {
    pub phase: Phase,
    pub occurred_at: DateTime<Utc>,
}

impl CountdownExpired {
    pub fn new(phase: Phase) -> Self {
        Self {
            phase,
            occurred_at: Utc::now(),
        }
    }
}

impl Event for CountdownExpired {
    fn event_type(&self) -> &'static str {
        "CountdownExpired"
    }

    fn aggregate_id(&self) -> String {
        "timer".to_string()
    }

    fn version(&self) -> u64 {
        1
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn clone_box(&self) -> Box<dyn Event> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
