use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::{EventHandler, TimerTickService};
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskResetHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
}

impl TaskResetHandler {
    pub fn new(emitter: Arc<dyn Emitter>, timer_srv: Arc<TimerTickService>) -> Self {
        TaskResetHandler { emitter, timer_srv }
    }
}

#[async_trait]
impl EventHandler for TaskResetHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskReset>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_reset = event
            .as_any()
            .downcast_ref::<domain::TaskReset>()
            .ok_or(domain::Error::EventHandlingError {
            message: format!("Failed to reset task"),
        })?;


        self.timer_srv.load_state().await?;

        self.timer_srv
            .stop_timer_tick_loop()
            .await
            .map_err(|e| domain::Error::EventHandlingError {
                message: format!("Failed to stop timer tick loop: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::task::RESET_TASK,
                json!(task_reset),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task reset event: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::task::LIST_UPDATED,
                json!(task_reset),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task list updated event: {e}"),
            })?;
        Ok(())
    }
}
