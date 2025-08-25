use crate::adapters::EventHandler;
use async_trait::async_trait;
use domain::{Event, Result};
use std::any::TypeId;
use tauri::{AppHandle, Emitter};

pub struct AutomaticTaskCyclingCompletedHandler {
    app_handle: AppHandle,
}

impl AutomaticTaskCyclingCompletedHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        AutomaticTaskCyclingCompletedHandler { app_handle }
    }
}

#[async_trait]
impl EventHandler for AutomaticTaskCyclingCompletedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::AutomaticTaskCyclingCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_cycling = event
            .as_any()
            .downcast_ref::<domain::AutomaticTaskCyclingCompleted>();

        self.app_handle
            .emit(domain::event_names::task::ACTIVE_CHANGED, task_cycling)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit automatic task cycling completed event: {e}"
                ),
            })?;
        Ok(())
    }
}
