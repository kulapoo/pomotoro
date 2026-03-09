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
        let _timer_reset = event
            .as_any()
            .downcast_ref::<domain::TimerReset>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to reset timer".to_string(),
            })?;

        self.timer_srv.load_state().await?;

        let state_json = self
            .timer_srv
            .with_timer(|t| {
                log::info!("{:?} timer reset", t);
                json!(t.state())
            })
            .await;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::RESET,
                state_json.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer reset event: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::STATUS_CHANGED,
                state_json,
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit timer status changed event: {e}"
                ),
            })?;

        Ok(())
    }
}
