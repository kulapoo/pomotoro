use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskUpdatedHandler {
    emitter: Arc<dyn Emitter>,
}

impl TaskUpdatedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        TaskUpdatedHandler { emitter }
    }
}

#[async_trait]
impl EventHandler for TaskUpdatedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskUpdated>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_updated = event.as_any().downcast_ref::<domain::TaskUpdated>();

        self.emitter
            .emit(domain::event_names::task::LIST_UPDATED, json!(task_updated))
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task updated event: {e}"),
            })?;
        Ok(())
    }
}
