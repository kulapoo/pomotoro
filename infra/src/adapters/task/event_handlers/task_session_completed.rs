use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskSessionCompletedHandler {
    emitter: Arc<dyn Emitter>,
}

impl TaskSessionCompletedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        TaskSessionCompletedHandler { emitter }
    }
}

#[async_trait]
impl EventHandler for TaskSessionCompletedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskSessionCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_session_completed = event
            .as_any()
            .downcast_ref::<domain::TaskSessionCompleted>();

        self.emitter
            .emit(
                domain::event_names::task::PROGRESS_UPDATED,
                json!(task_session_completed),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit task session completed event: {e}"
                ),
            })?;
        Ok(())
    }
}
