use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::{EventHandler, TimerTickService};
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TimerResetHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
}

impl TimerResetHandler {
    pub fn new(emitter: Arc<dyn Emitter>, timer_srv: Arc<TimerTickService>) -> Self {
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
            message: format!("Failed to reset timer"),
        })?;


        self.timer_srv.load_state().await?;

        self.timer_srv
            .stop_timer_tick_loop()
            .await
            .map_err(|e| domain::Error::EventHandlingError {
                message: format!("Failed to stop timer tick loop: {e}"),
            })?;

        self.timer_srv
            .start_timer_tick_loop(Some(timer_reset.timer_configuration.clone()), None)
            .await
            .map_err(|e| domain::Error::EventHandlingError {
                message: format!("Failed to start timer tick loop: {e}"),
            })?;


        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::RESET,
                json!(timer_reset),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer reset event: {e}"),
            })?;
        Ok(())
    }
}
