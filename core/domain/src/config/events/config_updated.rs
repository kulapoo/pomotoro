use crate::config::config::Config;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigUpdated {
    pub config: Config,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl ConfigUpdated {
    pub fn new(config: Config) -> Self {
        let now = Utc::now();
        Self {
            config,
            version: 1,
            occurred_at: now,
        }
    }
}

impl crate::Event for ConfigUpdated {
    fn event_type(&self) -> &'static str {
        "ConfigUpdated"
    }

    fn aggregate_id(&self) -> String {
        "config".to_string()
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
