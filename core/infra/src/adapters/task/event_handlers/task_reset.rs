use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

/// UI-only emitter for `TaskReset`.
///
/// Per the tick-loop ownership contract, this handler MUST NOT stop the tick
/// loop. The orchestrator that called `reset_task` owns the
/// `stop_timer_tick_loop` + `load_state` side effects. The previous
/// implementation raced with `TimerStartedHandler` on `cancel_handle` because
/// the event bus dispatches handlers on detached `tokio::spawn` tasks.
pub struct TaskResetHandler {
    emitter: Arc<dyn Emitter>,
}

impl TaskResetHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        TaskResetHandler { emitter }
    }
}

#[async_trait]
impl EventHandler for TaskResetHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskReset>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_reset = event
            .as_any()
            .downcast_ref::<domain::TaskReset>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to reset task".to_string(),
            })?;

        // The orchestrator that called `reset_task` is responsible for
        // stop_timer_tick_loop + load_state. This handler is a UI-only emitter.
        self.emitter
            .emit(
                domain::event_names::ui_listeners::task::TASK_RESET,
                json!(task_reset),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task reset event: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::task::LIST_UPDATED,
                json!(task_reset),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task list updated event: {e}"),
            })?;
        Ok(())
    }
}
