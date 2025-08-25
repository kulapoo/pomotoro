use crate::AudioConfig;
use crate::task::{config::Config, id::Id};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Created {
    pub task_id: Id,
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub tags: Vec<String>,
    pub config: Config,
    pub audio_config: AudioConfig,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl Created {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        task_id: Id,
        name: String,
        description: Option<String>,
        max_sessions: u8,
        tags: Vec<String>,
        config: Config,
        audio_config: AudioConfig,
        version: u64,
    ) -> Self {
        Self {
            task_id,
            name,
            description,
            max_sessions,
            tags,
            config,
            audio_config,
            version,
            occurred_at: Utc::now(),
        }
    }
}

impl crate::Event for Created {
    fn event_type(&self) -> &'static str {
        "TaskCreated"
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
