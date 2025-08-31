use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppExited {
    pub version: u64,
    pub app_version: String,
    pub app_exit_code: i32,
    pub exit_duration_ms: Option<u64>,
    pub app_terminated: bool,
    pub occurred_at: DateTime<Utc>,
}

impl AppExited {
    pub fn new(
        version: u64,
        app_version: String,
        app_exit_code: i32,
        exit_duration_ms: Option<u64>,
        app_terminated: bool,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        Self {
            version,
            app_version,
            app_exit_code,
            exit_duration_ms,
            app_terminated,
            occurred_at,
        }
    }
}

impl crate::Event for AppExited {
    fn event_type(&self) -> &'static str {
        "AppExited"
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
