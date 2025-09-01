use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskCreatedHandler {
    emitter: Arc<dyn Emitter>,
}

impl TaskCreatedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        TaskCreatedHandler { emitter }
    }
}

#[async_trait]
impl EventHandler for TaskCreatedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskCreated>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_created = event.as_any().downcast_ref::<domain::TaskCreated>();
        self.emitter
            .emit(domain::event_names::task::LIST_UPDATED, json!(task_created))
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task created event: {e}"),
            })?;
        Ok(())
    }
}
