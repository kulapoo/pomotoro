use async_trait::async_trait;
use domain::{Event, Result, PhaseCompleted};
use std::any::TypeId;
use tauri::Emitter;

use crate::adapters::events::EventHandler;

pub struct PhaseCompletedHandler {
    app_handle: tauri::AppHandle,
}

impl PhaseCompletedHandler {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

#[async_trait]
impl EventHandler for PhaseCompletedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<PhaseCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(phase_completed) = event.as_any().downcast_ref::<PhaseCompleted>() {
            self.app_handle
                .emit("timer:phase_completed", phase_completed.clone())
                .map_err(|e| domain::Error::RepositoryError {
                    message: format!("Failed to emit phase completed event: {}", e),
                })?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "PhaseCompletedHandler"
    }
}

impl From<PhaseCompletedHandler> for Box<dyn EventHandler> {
    fn from(handler: PhaseCompletedHandler) -> Self {
        Box::new(handler)
    }
}