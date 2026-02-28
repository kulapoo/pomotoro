use crate::task::id::Id;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Updated {
    pub task_id: Id,
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_sessions: Option<u8>,
    pub tags: Option<Vec<String>>,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl Updated {
    pub fn new(
        task_id: Id,
        name: Option<String>,
        description: Option<String>,
        max_sessions: Option<u8>,
        tags: Option<Vec<String>>,
        version: u64,
    ) -> Self {
        Self {
            task_id,
            name,
            description,
            max_sessions,
            tags,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for Updated {
    fn event_type(&self) -> &'static str {
        "TaskUpdated"
    }

    fn aggregate_id(&self) -> String {
        self.task_id.to_string()
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
