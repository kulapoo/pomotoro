use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::{EventHandler, TimerTickService};
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

/// UI-only emitter for `TimerReset`.
///
/// Per the tick-loop ownership contract, this handler MUST NOT stop the tick
/// loop. The orchestrator that called `reset_timer_to_idle` (or equivalent)
/// owns the stop call. The previous implementation raced with
/// `TimerStartedHandler` on `cancel_handle` because the event bus dispatches
/// handlers on detached `tokio::spawn` tasks.
pub struct TimerResetHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
}

impl TimerResetHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        timer_srv: Arc<TimerTickService>,
    ) -> Self {
        TimerResetHandler { emitter, timer_srv }
    }
}

#[async_trait]
impl EventHandler for TimerResetHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TimerReset>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let timer_reset = event
            .as_any()
            .downcast_ref::<domain::TimerReset>()
            .ok_or(domain::Error::EventHandlingError {
            message: "Failed to reset timer".to_string(),
        })?;

        let task_id = timer_reset.task_id.to_string();
        self.timer_srv.load_state().await?;
        // Read-only: format the current timer state for the UI. The
        // orchestrator has already stopped the loop and refreshed state.
        let state_json = self
            .timer_srv
            .with_timer(|t| {
                log::info!("{:?} timer reset", t);
                json!(t.state())
            })
            .await;

        let payload = json!({
            "task_id": task_id,
            "state": state_json,
        });

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::RESET,
                payload.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer reset event: {e}"),
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
