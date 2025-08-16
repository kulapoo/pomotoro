use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::{TaskId, TaskConfig, AudioConfig};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskCreated {
    pub task_id: TaskId,
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub tags: Vec<String>,
    pub config: TaskConfig,
    pub audio_config: AudioConfig,
    pub version: u64,
    pub occurred_at: DateTime<Utc>,
}

impl TaskCreated {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        task_id: TaskId,
        name: String,
        description: Option<String>,
        max_sessions: u8,
        tags: Vec<String>,
        config: TaskConfig,
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

impl crate::Event for TaskCreated {
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
