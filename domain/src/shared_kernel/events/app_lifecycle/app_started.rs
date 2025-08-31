use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppStarted {
    pub version: u64,
    pub app_version: String,
    pub config_loaded: bool,
    pub default_task_created: bool,
    pub timer_auto_started: bool,
    pub startup_duration_ms: Option<u64>,
    pub occurred_at: DateTime<Utc>,
}

impl AppStarted {
    pub fn new(
        version: u64,
        app_version: String,
        config_loaded: bool,
        default_task_created: bool,
        timer_auto_started: bool,
        startup_duration_ms: Option<u64>,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        Self {
            version,
            app_version,
            config_loaded,
            default_task_created,
            timer_auto_started,
            startup_duration_ms,
            occurred_at,
        }
    }
}

impl crate::Event for AppStarted {
    fn event_type(&self) -> &'static str {
        "AppStarted"
    }

    fn aggregate_id(&self) -> String {
        "app".to_string()
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
