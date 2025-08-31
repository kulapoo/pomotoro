use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskSwitchWorkflowCompletedHandler {
    emitter: Arc<dyn Emitter>,
}

impl TaskSwitchWorkflowCompletedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        TaskSwitchWorkflowCompletedHandler { emitter }
    }
}

#[async_trait]
impl EventHandler for TaskSwitchWorkflowCompletedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskSwitchWorkflowCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_switch = event
            .as_any()
            .downcast_ref::<domain::TaskSwitchWorkflowCompleted>();

        self.emitter
            .emit(domain::event_names::task::ACTIVE_CHANGED, json!(task_switch))
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit task switch workflow completed event: {e}"
                ),
            })?;
        Ok(())
    }
}
