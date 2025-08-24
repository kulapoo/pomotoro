use std::any::TypeId;
use async_trait::async_trait;
use domain::{Event, Result};
use tauri::{AppHandle, Emitter};
use crate::adapters::EventHandler;

pub struct SessionTransitionCompletedHandler {
    app_handle: AppHandle,
}

impl SessionTransitionCompletedHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        SessionTransitionCompletedHandler { app_handle }
    }
}

#[async_trait]
impl EventHandler for SessionTransitionCompletedHandler {

    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::SessionTransitionCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let session_transition = event.as_any().downcast_ref::<domain::SessionTransitionCompleted>();

        self.app_handle.emit(domain::events::task::PROGRESS_UPDATED, session_transition)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit session transition completed event: {e}")
            })?;
        Ok(())
    }
}