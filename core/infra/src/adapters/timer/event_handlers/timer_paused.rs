use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::{EventHandler, TimerTickService};
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TimerPausedHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
}

impl TimerPausedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        timer_srv: Arc<TimerTickService>,
    ) -> Self {
        TimerPausedHandler { emitter, timer_srv }
    }
}

#[async_trait]
impl EventHandler for TimerPausedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TimerPaused>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let timer_paused = event
            .as_any()
            .downcast_ref::<domain::TimerPaused>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to pause timer".to_string(),
            })?;

        let task_id = timer_paused.task_id.to_string();

        self.timer_srv.load_state().await?;
        // The orchestrator that called `pause_timer_phase` is responsible for
        // stop_timer_tick_loop + load_state. This handler is a UI-only emitter.
        let state_json = self.timer_srv.with_timer(|t| json!(t.state())).await;

        let payload = json!({
            "task_id": task_id,
            "state": state_json,
        });

        self.emitter
            .emit(domain::event_names::ui_listeners::timer::PAUSE, payload)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer paused event: {e}"),
            })?;
        Ok(())
    }
}
