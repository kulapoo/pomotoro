use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskDeletedHandler {
    emitter: Arc<dyn Emitter>,
}

impl TaskDeletedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        TaskDeletedHandler { emitter }
    }
}

#[async_trait]
impl EventHandler for TaskDeletedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskDeleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_deleted = event.as_any().downcast_ref::<domain::TaskDeleted>();

        self.emitter
            .emit(domain::event_names::task::TASK_DELETED, json!(task_deleted))
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task updated event: {e}"),
            })?;
        self.emitter
            .emit(domain::event_names::task::LIST_UPDATED, json!(task_deleted))
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task updated event: {e}"),
            })?;
        Ok(())
    }
}
