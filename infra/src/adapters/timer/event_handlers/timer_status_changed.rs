use async_trait::async_trait;
use domain::{Event, Result, TimerStatusChanged};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::events::EventHandler;

pub struct TimerStatusChangedHandler {
    emitter: Arc<dyn Emitter>,
}

impl TimerStatusChangedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        Self { emitter }
    }
}

#[async_trait]
impl EventHandler for TimerStatusChangedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<TimerStatusChanged>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(status_changed) =
            event.as_any().downcast_ref::<TimerStatusChanged>()
        {
            self.emitter
                .emit("timer:status_changed", json!(status_changed.clone()))
                .map_err(|e| domain::Error::RepositoryError {
                    message: format!(
                        "Failed to emit timer status changed event: {e}"
                    ),
                })?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "TimerStatusChangedHandler"
    }
}

impl From<TimerStatusChangedHandler> for Box<dyn EventHandler> {
    fn from(handler: TimerStatusChangedHandler) -> Self {
        Box::new(handler)
    }
}
