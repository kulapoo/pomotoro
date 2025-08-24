use std::any::TypeId;
use async_trait::async_trait;
use domain::{Event, Result};
use tauri::{AppHandle, Emitter};
use crate::adapters::EventHandler;

pub struct TaskCreatedHandler {
    app_handle: AppHandle,
}

impl TaskCreatedHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        TaskCreatedHandler { app_handle }
    }
}

#[async_trait]
impl EventHandler for TaskCreatedHandler {

    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskCreated>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_created = event.as_any().downcast_ref::<domain::TaskCreated>();

        self.app_handle.emit(domain::event_names::task::LIST_UPDATED, task_created)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task created event: {e}")
            })?;
        Ok(())
    }
}