use async_trait::async_trait;
use domain::{Event, Result, TimerTick};
use std::any::TypeId;
use tauri::Emitter;

use crate::adapters::events::EventHandler;

pub struct TimerTickHandler {
    app_handle: tauri::AppHandle,
}

impl TimerTickHandler {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
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
            self.app_handle
                .emit("timer:tick", timer_tick.clone())
                .map_err(|e| domain::Error::RepositoryError {
                    message: format!("Failed to emit timer tick event: {}", e),
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