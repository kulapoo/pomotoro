use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TimerStartedHandler {
    emitter: Arc<dyn Emitter>,
}

impl TimerStartedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        TimerStartedHandler { emitter }
    }
}

#[async_trait]
impl EventHandler for TimerStartedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TimerStarted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let timer_started =
            event.as_any().downcast_ref::<domain::TimerStarted>();

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::STATUS_CHANGED,
                json!(timer_started),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer started event: {e}"),
            })?;
        Ok(())
    }
}
