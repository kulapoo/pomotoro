use serde::{Deserialize, Serialize};

use crate::{timer::Phase, TimerConfiguration};
use crate::TimerId;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reset {
    pub timer_id: TimerId,
    pub phase: Phase,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
    pub timer_configuration: TimerConfiguration,
}

impl Reset {
    pub fn new(
        timer_id: TimerId,
        phase: Phase,
        version: u64,
        timer_configuration: TimerConfiguration,
    ) -> Self {
        Self {
            timer_id,
            phase,
            version,
            occurred_at: Utc::now(),
            timer_configuration,
        }
    }
}

impl crate::Event for Reset {
    fn event_type(&self) -> &'static str {
        "Reset"
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
