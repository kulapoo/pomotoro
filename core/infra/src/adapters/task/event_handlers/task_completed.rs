use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

/// Emits the `task:list_updated` and `timer:status_changed` UI events when a
/// task is completed.
///
/// This handler is a pure UI side-effect translator. It intentionally does
/// NOT stop or reset the timer — that orchestration belongs in the use case
/// / command layer that has full context (manual complete vs. auto-cycle).
/// Resetting the timer here would race with the cycling logic in
/// `progress_phase`, which emits `TaskCompleted` at the trailing-break
/// boundary while the timer is mid-transition.
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
        let task_completed = event
            .as_any()
            .downcast_ref::<domain::TaskCompleted>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to complete task".to_string(),
            })?;

        self.emitter
            .emit(
                domain::event_names::task::LIST_UPDATED,
                json!(task_completed),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task completed event: {e}"),
            })?;

        // self.emitter
        //     .emit(
        //         domain::event_names::task::TASK_COMPLETED,
        //         json!(task_completed),
        //     )
        //     .map_err(|e| domain::Error::EventPublishingError {
        //         message: format!("Failed to emit task completed event: {e}"),
        //     })?;

        Ok(())
    }
}
