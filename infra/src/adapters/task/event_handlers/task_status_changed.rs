use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskStatusChangedHandler {
    emitter: Arc<dyn Emitter>,
}

impl TaskStatusChangedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        TaskStatusChangedHandler { emitter }
    }
}

#[async_trait]
impl EventHandler for TaskStatusChangedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskStatusChanged>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_status_changed =
            event.as_any().downcast_ref::<domain::TaskStatusChanged>();

        self.emitter
            .emit(domain::event_names::task::LIST_UPDATED, json!(task_status_changed))
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit task status changed event: {e}"
                ),
            })?;
        Ok(())
    }
}
