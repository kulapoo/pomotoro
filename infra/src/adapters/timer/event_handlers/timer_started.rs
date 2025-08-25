use crate::adapters::EventHandler;
use async_trait::async_trait;
use domain::{Event, Result};
use std::any::TypeId;
use tauri::{AppHandle, Emitter};

pub struct TimerStartedHandler {
    app_handle: AppHandle,
}

impl TimerStartedHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        TimerStartedHandler { app_handle }
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

        self.app_handle
            .emit(
                domain::event_names::ui_listeners::app::APP_STARTED,
                timer_started,
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer started event: {e}"),
            })?;
        Ok(())
    }
}
