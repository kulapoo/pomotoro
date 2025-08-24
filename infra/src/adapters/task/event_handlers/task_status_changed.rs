use std::any::TypeId;
use async_trait::async_trait;
use domain::{Event, Result};
use tauri::{AppHandle, Emitter};
use crate::adapters::EventHandler;

pub struct TaskStatusChangedHandler {
    app_handle: AppHandle,
}

impl TaskStatusChangedHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        TaskStatusChangedHandler { app_handle }
    }
}

#[async_trait]
impl EventHandler for TaskStatusChangedHandler {

    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskStatusChanged>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_status_changed = event.as_any().downcast_ref::<domain::TaskStatusChanged>();

        self.app_handle.emit(domain::events::task::LIST_UPDATED, task_status_changed)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task status changed event: {e}")
            })?;
        Ok(())
    }
}