use crate::adapters::{
    TimerTickService,
    events::{EventHandler, app_emitter::Emitter},
};
use async_trait::async_trait;
use domain::{
    ConfigRepository, EventPublisher, Phase, TimerRepository,
    event_names::ui_listeners, timer::events::CountdownExpired,
};
use domain::{Error, TaskRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;
use usecases::timer::{
    complete_timer_phase, pause_timer_phase, reset_timer_phase,
};

/// Event handler that triggers phase completion when countdown naturally expires
/// and handles auto-start of next phase based on configuration
pub struct CountdownExpiredHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
    task_repository: Arc<dyn TaskRepository + Send + Sync>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    timer_repository: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
}

impl CountdownExpiredHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        timer_srv: Arc<TimerTickService>,
        task_repository: Arc<dyn TaskRepository + Send + Sync>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
        timer_repository: Arc<dyn TimerRepository + Send + Sync>,
        event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    ) -> Self {
        Self {
            emitter,
            timer_srv,
            task_repository,
            config_repository,
            timer_repository,
            event_publisher,
        }
    }
}

#[async_trait]
impl EventHandler for CountdownExpiredHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<CountdownExpired>()
    }

    async fn handle(&self, event: Box<dyn domain::Event>) -> Result<(), Error> {
        let countdown_expired = event
            .as_any()
            .downcast_ref::<CountdownExpired>()
            .ok_or_else(|| Error::RepositoryError {
                message: "Failed to downcast to CountdownExpired event"
                    .to_string(),
            })?;

        // Get configuration to check auto-start settings
        let config = self.config_repository.get_config().await?;

        let should_auto_start = match countdown_expired.phase {
            Phase::Work => {
                // Work phase expired, check if we should auto-start break
                config.general.auto_start_breaks
            }
            Phase::ShortBreak | Phase::LongBreak => {
                config.general.auto_start_work_after_break
            }
        };
        let task_id = countdown_expired.task_id;
        if should_auto_start {
            let (task, _timer, next_phase) = complete_timer_phase(
                task_id,
                self.task_repository.clone(),
                self.timer_repository.clone(),
                self.event_publisher.clone(),
            )
            .await?;

            // Load the state after complete_timer_phase saves to repository
            self.timer_srv.load_state().await?;

            // Reset the timer phase using the usecase (business operation)
            reset_timer_phase(
                task.id(),
                self.task_repository.clone(),
                self.timer_repository.clone(),
                self.event_publisher.clone(),
            )
            .await?;

            let timer_config = task.config().timer.clone();

            if !(task.is_completed() && next_phase == Phase::Work) {
                // Start the timer tick loop (infrastructure concern)
                self.timer_srv
                    .start_timer_tick_loop(Some(timer_config), None)
                    .await
                    .map_err(|e| domain::Error::RepositoryError {
                        message: format!("Failed to auto-start timer: {}", e),
                    })?;

                // Get the current timer state AFTER all operations complete (avoids cloning entire Timer)
                let state_json =
                    self.timer_srv.with_timer(|t| json!(t.state())).await;

                self.emitter
                    .emit(
                        domain::event_names::ui_listeners::timer::PHASE_COMPLETED,
                        state_json,
                    )
                    .map_err(|e| domain::Error::EventPublishingError {
                        message: format!(
                            "Failed to emit timer phase completed event: {e}"
                        ),
                    })?;
            }

            self.emitter
                .emit(
                    domain::event_names::ui_listeners::task::PROGRESS_UPDATED,
                    json!(task),
                )
                .map_err(|e| domain::Error::EventPublishingError {
                    message: format!(
                        "Failed to emit task progress updated event: {e}"
                    ),
                })?;

            // log::info!(
            //     "Timer auto-started in phase {:?} with {} seconds remaining",
            //     timer.state().phase(),
            //     timer.state().remaining_seconds()
            // );
        } else {
            log::debug!(
                "Auto-start not enabled for transition from {:?} phase",
                countdown_expired.phase
            );
            // Complete the phase first (timer is in LongBreak/ShortBreak/Working
            // state in the repo — can_transition allows CompletePhase from these).
            // Pausing first would move the repo to Paused, from which CompletePhase
            // is not allowed, causing an InvalidStateTransition error.
            complete_timer_phase(
                task_id,
                self.task_repository.clone(),
                self.timer_repository.clone(),
                self.event_publisher.clone(),
            )
            .await?;
            // Now pause the next phase so the UI shows "Paused" and the user
            // must manually resume to start the new phase.
            let timer = pause_timer_phase(
                task_id,
                self.task_repository.clone(),
                self.timer_repository.clone(),
                self.event_publisher.clone(),
            )
            .await?;

            self.timer_srv.load_state().await?;

            self.emitter
                .emit(ui_listeners::timer::STATUS_CHANGED, json!(timer.state()))
                .map_err(|e| domain::Error::RepositoryError {
                    message: format!(
                        "Failed to emit timer status changed event: {e}"
                    ),
                })?;
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "CountdownExpiredHandler"
    }
}
