use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskCompletedHandler {
    emitter: Arc<dyn Emitter>,
}

impl TaskCompletedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        TaskCompletedHandler { emitter }
    }
}

#[async_trait]
impl EventHandler for TaskCompletedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_completed =
            event.as_any().downcast_ref::<domain::TaskCompleted>();

        self.emitter
            .emit(domain::event_names::task::LIST_UPDATED, json!(task_completed))
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task completed event: {e}"),
            })?;
        Ok(())
    }
}
