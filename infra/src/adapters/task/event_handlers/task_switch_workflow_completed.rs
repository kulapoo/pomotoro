use crate::adapters::EventHandler;
use async_trait::async_trait;
use domain::{Event, Result};
use std::any::TypeId;
use tauri::{AppHandle, Emitter};

pub struct TaskSwitchWorkflowCompletedHandler {
    app_handle: AppHandle,
}

impl TaskSwitchWorkflowCompletedHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        TaskSwitchWorkflowCompletedHandler { app_handle }
    }
}

#[async_trait]
impl EventHandler for TaskSwitchWorkflowCompletedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskSwitchWorkflowCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_switch = event
            .as_any()
            .downcast_ref::<domain::TaskSwitchWorkflowCompleted>();

        self.app_handle
            .emit(domain::event_names::task::ACTIVE_CHANGED, task_switch)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit task switch workflow completed event: {e}"
                ),
            })?;
        Ok(())
    }
}
