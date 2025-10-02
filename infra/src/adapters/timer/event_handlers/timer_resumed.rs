use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::{EventHandler, TimerTickService};
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

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
                message: format!("Failed to resume timer"),
            })?;

        // Load the current state to get the full TimerState
        self.timer_srv.load_state().await?;

        let _ = self.timer_srv.start_timer_tick_loop(None).await;

        // Get the current timer state to emit to UI
        let timer = self.timer_srv.get_current_timer().await;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::RESUME,
                json!(timer),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer resumed event: {e}"),
            })?;
        Ok(())
    }
}
