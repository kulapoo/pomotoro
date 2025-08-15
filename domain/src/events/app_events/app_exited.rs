use serde::{Deserialize, Serialize};

use crate::DomainEvent;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppExited {
    pub version: u64,
    pub app_version: String,
    pub app_exit_code: i32,
    pub exit_duration_ms: Option<u64>,
    pub app_terminated: bool,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

impl AppExited {
    pub fn new(
        version: u64,
        app_version: String,
        app_exit_code: i32,
        exit_duration_ms: Option<u64>,
        app_terminated: bool,
        occurred_at: chrono::DateTime<chrono::Utc>,
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

impl DomainEvent for AppExited {
    fn event_type(&self) -> &'static str {
        "AppExited"
    }

    fn aggregate_id(&self) -> String {
        "app".to_string()
    }
    
    fn version(&self) -> u64 {
        self.version
    }
    
    fn occurred_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.occurred_at
    }
    
    fn clone_box(&self) -> Box<dyn DomainEvent> {
        Box::new(self.clone())
    }
    
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}