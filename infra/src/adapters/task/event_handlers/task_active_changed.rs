use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskActiveChangedHandler {
    emitter: Arc<dyn Emitter>,
}

impl TaskActiveChangedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        TaskActiveChangedHandler { emitter }
    }
}

#[async_trait]
impl EventHandler for TaskActiveChangedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskActiveChanged>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_switch = event
            .as_any()
            .downcast_ref::<domain::TaskActiveChanged>();

        self.emitter
            .emit(
                domain::event_names::task::ACTIVE_CHANGED,
                json!(task_switch),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit task active changed event: {e}"
                ),
            })?;
        Ok(())
    }
}
