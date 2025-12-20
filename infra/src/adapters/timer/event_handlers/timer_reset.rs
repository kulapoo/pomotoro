use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::{EventHandler, TimerTickService};
use async_trait::async_trait;
use domain::{Event, EventPublisher, Result, TaskRepository, TimerRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;
use usecases::timer::pause_timer_phase;

pub struct TimerResetHandler {
    emitter: Arc<dyn Emitter>,
    task_repo: Arc<dyn TaskRepository>,
    timer_repo: Arc<dyn TimerRepository>,
    event_publisher: Arc<dyn EventPublisher>,
    timer_srv: Arc<TimerTickService>,
}

impl TimerResetHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        task_repo: Arc<dyn TaskRepository>,
        timer_repo: Arc<dyn TimerRepository>,
        event_publisher: Arc<dyn EventPublisher>,
        timer_srv: Arc<TimerTickService>,
    ) -> Self {
        TimerResetHandler {
            emitter,
            task_repo,
            timer_repo,
            event_publisher,
            timer_srv,
        }
    }
}

#[async_trait]
impl EventHandler for TimerResetHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TimerReset>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let timer_reset = event
            .as_any()
            .downcast_ref::<domain::TimerReset>()
            .ok_or(domain::Error::EventHandlingError {
            message: "Failed to reset timer".to_string(),
        })?;

        self.timer_srv.stop_timer_tick_loop().await.map_err(|e| {
            domain::Error::EventHandlingError {
                message: format!("Failed to stop timer tick loop: {e}"),
            }
        })?;

        let timer = pause_timer_phase(
            timer_reset.task_id,
            self.task_repo.clone(),
            self.timer_repo.clone(),
            self.event_publisher.clone(),
        )
        .await?;

        log::info!("{:?} timer reset", timer);

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::RESET,
                json!(timer.state()),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer reset event: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::timer::STATUS_CHANGED,
                json!(timer.state()),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit timer status changed event: {e}"
                ),
            })?;

        Ok(())
    }
}
