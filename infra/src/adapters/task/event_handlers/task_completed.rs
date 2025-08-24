use std::any::TypeId;
use async_trait::async_trait;
use domain::{Event, Result};
use tauri::{AppHandle, Emitter};
use crate::adapters::EventHandler;

pub struct TaskCompletedHandler {
    app_handle: AppHandle,
}

impl TaskCompletedHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        TaskCompletedHandler { app_handle }
    }
}

#[async_trait]
impl EventHandler for TaskCompletedHandler {

    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_completed = event.as_any().downcast_ref::<domain::TaskCompleted>();

        self.app_handle.emit(domain::events::task::LIST_UPDATED, task_completed)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task completed event: {e}")
            })?;
        Ok(())
    }
}