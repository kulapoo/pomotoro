use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::{EventHandler, TimerTickService};
use async_trait::async_trait;
use domain::{Event, Result, TaskId, TaskRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TimerStartedHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
    task_repository: Arc<dyn TaskRepository>,
}

impl TimerStartedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        timer_srv: Arc<TimerTickService>,
        task_repository: Arc<dyn TaskRepository>,
    ) -> Self {
        TimerStartedHandler {
            emitter,
            timer_srv,
            task_repository,
        }
    }
}

#[async_trait]
impl EventHandler for TimerStartedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TimerStarted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let timer_started = event
            .as_any()
            .downcast_ref::<domain::TimerStarted>()
            .ok_or(domain::Error::EventHandlingError {
                message: format!("Failed to start timer tick loop"),
            })?;
        let task_id = timer_started.active_entity_id.ok_or(
            domain::Error::InvalidTaskParams {
                message: "missing task id".into(),
            },
        )?;

        let task = self.task_repository.get_by_id(task_id).await?;

        self.timer_srv
            .start_timer_tick_loop(task.as_ref())
            .await
            .map_err(|e| domain::Error::EventHandlingError {
                message: format!("Failed to start timer tick loop: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::STATUS_CHANGED,
                json!(timer_started),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer started event: {e}"),
            })?;

        Ok(())
    }
}
