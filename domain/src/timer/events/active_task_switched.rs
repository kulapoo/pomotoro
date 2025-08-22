use serde::{Deserialize, Serialize};
use crate::timer::Phase;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActiveTaskSwitched {
    pub old_entity_id: Option<String>,
    pub new_entity_id: Option<String>,
    pub phase: Phase,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl ActiveTaskSwitched {
    pub fn new(
        old_entity_id: Option<String>,
        new_entity_id: Option<String>,
        phase: Phase,
        version: u64,
    ) -> Self {
        Self {
            old_entity_id,
            new_entity_id,
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
        self.new_entity_id
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
