use crate::timer::Phase;
use crate::TimerId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveTaskSwitched {
    pub timer_id: TimerId,
    pub phase: Phase,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl ActiveTaskSwitched {
    pub fn new(
        timer_id: TimerId,
        phase: Phase,
        version: u64,
    ) -> Self {
        Self {
            timer_id,
            phase,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for ActiveTaskSwitched {
    fn event_type(&self) -> &'static str {
        "ActiveTaskSwitched"
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
