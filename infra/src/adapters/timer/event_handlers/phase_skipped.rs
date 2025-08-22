use async_trait::async_trait;
use domain::{Event, Result, PhaseSkipped};
use std::any::TypeId;
use tauri::Emitter;

use crate::adapters::events::EventHandler;

pub struct PhaseSkippedHandler {
    app_handle: tauri::AppHandle,
}

impl PhaseSkippedHandler {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }
}

#[async_trait]
impl EventHandler for PhaseSkippedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<PhaseSkipped>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(phase_skipped) = event.as_any().downcast_ref::<PhaseSkipped>() {
            self.app_handle
                .emit("timer:phase_skipped", phase_skipped.clone())
                .map_err(|e| domain::Error::RepositoryError {
                    message: format!("Failed to emit phase skipped event: {}", e),
                })?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "PhaseSkippedHandler"
    }
}

impl From<PhaseSkippedHandler> for Box<dyn EventHandler> {
    fn from(handler: PhaseSkippedHandler) -> Self {
        Box::new(handler)
    }
}