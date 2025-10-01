use serde::{Deserialize, Serialize};

use crate::TimerConfiguration;
use crate::TimerId;
use crate::timer::Phase;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resumed {
    pub timer_id: TimerId,
    pub phase: Phase,
    pub remaining_seconds: u32,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
    pub config: TimerConfiguration,
}

impl Resumed {
    pub fn new(
        timer_id: TimerId,
        phase: Phase,
        remaining_seconds: u32,
        version: u64,
        config: TimerConfiguration,
    ) -> Self {
        Self {
            timer_id,
            phase,
            remaining_seconds,
            version,
            occurred_at: Utc::now(),
            config,
        }
    }
}

impl crate::Event for Resumed {
    fn event_type(&self) -> &'static str {
        "Resumed"
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
