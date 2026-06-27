use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::{EventHandler, TimerTickService};
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

/// UI-only emitter for `TimerStarted`.
///
/// Per the tick-loop ownership contract on `TimerTickService::start_timer_tick_loop`,
/// this handler MUST NOT drive the tick loop. The orchestrator that called
/// `start_timer_phase` (Tauri command, tray handler, or `CountdownExpiredHandler`)
/// is responsible for calling `start_timer_tick_loop` directly. Routing the
/// side effect through this handler caused the auto-advance race, because
/// `InMemoryEventBus::publish` spawns handlers on detached `tokio::spawn`
/// tasks whose order relative to `TimerResetHandler` is non-deterministic.
pub struct TimerStartedHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
}

impl TimerStartedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        timer_srv: Arc<TimerTickService>,
    ) -> Self {
        TimerStartedHandler { emitter, timer_srv }
    }
}

#[async_trait]
impl EventHandler for TimerStartedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TimerStarted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let _timer_started = event
            .as_any()
            .downcast_ref::<domain::TimerStarted>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to start timer tick loop".to_string(),
            })?;

        // Read-only access to format the UI payload. No mutation of
        // cancel_handle. The orchestrator has already started the loop.
        let state_json = self.timer_srv.with_timer(|t| json!(t.state())).await;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::STATUS_CHANGED,
                state_json.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer started event: {e}"),
            })?;

        self.emitter
            .emit(domain::event_names::ui_listeners::timer::START, state_json)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer started event: {e}"),
            })?;

        Ok(())
    }
}
