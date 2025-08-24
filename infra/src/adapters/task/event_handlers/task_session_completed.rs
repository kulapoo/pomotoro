use std::any::TypeId;
use async_trait::async_trait;
use domain::{Event, Result};
use tauri::{AppHandle, Emitter};
use crate::adapters::EventHandler;

pub struct TaskSessionCompletedHandler {
    app_handle: AppHandle,
}

impl TaskSessionCompletedHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        TaskSessionCompletedHandler { app_handle }
    }
}

#[async_trait]
impl EventHandler for TaskSessionCompletedHandler {

    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskSessionCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_session_completed = event.as_any().downcast_ref::<domain::TaskSessionCompleted>();

        self.app_handle.emit(domain::event_names::task::PROGRESS_UPDATED, task_session_completed)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task session completed event: {e}")
            })?;
        Ok(())
    }
}