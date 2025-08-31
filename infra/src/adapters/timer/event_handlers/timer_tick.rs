use async_trait::async_trait;
use domain::{Event, Result, TimerTick, event_names::ui_listeners};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::events::EventHandler;

pub struct TimerTickHandler {
    emitter: Arc<dyn Emitter>,
}

impl TimerTickHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        Self { emitter }
    }
}

#[async_trait]
impl EventHandler for TimerTickHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<TimerTick>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(timer_tick) = event.as_any().downcast_ref::<TimerTick>() {
            // Emit the timer tick event to the frontend
            self.emitter
                .emit(ui_listeners::timer::TICK, json!(timer_tick.clone()))
                .map_err(|e| domain::Error::RepositoryError {
                    message: format!("Failed to emit timer tick event: {e}"),
                })?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "TimerTickHandler"
    }
}

impl From<TimerTickHandler> for Box<dyn EventHandler> {
    fn from(handler: TimerTickHandler) -> Self {
        Box::new(handler)
    }
}
