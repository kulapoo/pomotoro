use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::{EventHandler, TimerTickService};
use async_trait::async_trait;
use domain::{Event, Result, TaskRepository};
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
                message: "Failed to start timer tick loop".to_string(),
            })?;
        let task_id = timer_started.task_id;

        let task = self.task_repository.get_by_id(task_id).await?;

        let timer_config = task.as_ref().map(|t| t.config().timer.clone());

        self.timer_srv
            .start_timer_tick_loop(timer_config, Some(task_id))
            .await
            .map_err(|e| domain::Error::EventHandlingError {
                message: format!("Failed to start timer tick loop: {e}"),
            })?;

        // Get only the timer state to emit to UI (avoids cloning entire Timer)
        let state_json = self.timer_srv.with_timer(|t| json!(t.state())).await;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::STATUS_CHANGED,
                state_json,
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer started event: {e}"),
            })?;

        Ok(())
    }
}
