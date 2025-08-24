use std::any::TypeId;
use async_trait::async_trait;
use domain::{Event, Result};
use tauri::{AppHandle, Emitter};
use crate::adapters::EventHandler;

pub struct TaskUpdatedHandler {
    app_handle: AppHandle,
}

impl TaskUpdatedHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        TaskUpdatedHandler { app_handle }
    }
}

#[async_trait]
impl EventHandler for TaskUpdatedHandler {

    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskUpdated>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_updated = event.as_any().downcast_ref::<domain::TaskUpdated>();

        self.app_handle.emit(domain::events::task::LIST_UPDATED, task_updated)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task updated event: {e}")
            })?;
        Ok(())
    }
}