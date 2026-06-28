use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::{EventHandler, TimerTickService};
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

/// UI-only emitter for `TimerResumed`.
///
/// Per the tick-loop ownership contract on
/// `TimerTickService::start_timer_tick_loop`, this handler MUST NOT drive the
/// tick loop. The orchestrator that called `resume_timer_phase` is responsible
/// for calling `start_timer_tick_loop` directly. This handler is read-only
/// with respect to `cancel_handle`.
pub struct TimerResumedHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
}

impl TimerResumedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        timer_srv: Arc<TimerTickService>,
    ) -> Self {
        TimerResumedHandler { emitter, timer_srv }
    }
}

#[async_trait]
impl EventHandler for TimerResumedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TimerResumed>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let timer_resumed = event
            .as_any()
            .downcast_ref::<domain::TimerResumed>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to resume timer".to_string(),
            })?;

        let task_id = timer_resumed.task_id.to_string();

        self.timer_srv.load_state().await?;
        // Read-only access to format the UI payload. No mutation of
        // cancel_handle. The orchestrator has already started the loop.
        let state_json = self.timer_srv.with_timer(|t| json!(t.state())).await;

        let payload = json!({
            "task_id": task_id,
            "state": state_json,
        });

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::RESUME,
                payload.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer resumed event: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::STATUS_CHANGED,
                payload,
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit timer status changed event: {e}"
                ),
            })?;

        Ok(())
    }
}
