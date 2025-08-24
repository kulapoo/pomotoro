use std::any::TypeId;
use async_trait::async_trait;
use domain::{Event, Result};
use tauri::{AppHandle, Emitter};
use crate::adapters::EventHandler;

pub struct TaskCyclingExhaustedHandler {
    app_handle: AppHandle,
}

impl TaskCyclingExhaustedHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        TaskCyclingExhaustedHandler { app_handle }
    }
}

#[async_trait]
impl EventHandler for TaskCyclingExhaustedHandler {

    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskCyclingExhausted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_exhausted = event.as_any().downcast_ref::<domain::TaskCyclingExhausted>();

        self.app_handle.emit(domain::event_names::task::LIST_UPDATED, task_exhausted)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task cycling exhausted event: {e}")
            })?;
        Ok(())
    }
}