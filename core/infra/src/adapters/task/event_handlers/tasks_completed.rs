use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

/// UI-only emitter for `TasksCompleted` — fires when the last incomplete
/// task is completed (the whole list is done). Shows a single celebratory
/// toast and refreshes the task list.
pub struct TasksCompletedHandler {
    emitter: Arc<dyn Emitter>,
}

impl TasksCompletedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        TasksCompletedHandler { emitter }
    }
}

#[async_trait]
impl EventHandler for TasksCompletedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TasksCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let tasks_completed = event
            .as_any()
            .downcast_ref::<domain::TasksCompleted>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to handle tasks completed event".to_string(),
            })?;

        let payload = json!({
            "completed_task_ids": tasks_completed.completed_task_ids,
            "version": tasks_completed.version,
            "occurred_at": tasks_completed.occurred_at,
        });

        self.emitter
            .emit(
                domain::event_names::ui_listeners::task::TASKS_COMPLETED,
                payload.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit tasks completed event: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::task::LIST_UPDATED,
                payload,
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task list updated event: {e}"),
            })?;
        Ok(())
    }
}
