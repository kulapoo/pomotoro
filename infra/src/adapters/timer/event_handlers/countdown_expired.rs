use crate::adapters::{
    TimerTickService,
    events::{EventHandler, app_emitter::Emitter},
};
use async_trait::async_trait;
use domain::{
    ConfigRepository, EventPublisher, Phase, TimerRepository,
    timer::events::CountdownExpired,
};
use domain::{Error, TaskRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;
use usecases::timer::complete_timer_phase;

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

        // Get the task to access its timer configuration

        let should_auto_start = match countdown_expired.phase.clone() {
            Phase::Work => {
                // Work phase expired, check if we should auto-start break

                config.general.auto_start_breaks
            }
            Phase::ShortBreak | Phase::LongBreak => {
                config.general.auto_start_work_after_break
            }
        };
        if should_auto_start {
            let task_id = countdown_expired.task_id.clone();

            let (task, _timer) = complete_timer_phase(
                task_id,
                self.task_repository.clone(),
                self.timer_repository.clone(),
                self.event_publisher.clone(),
            )
            .await?;
            // log::info!(
            //     "Auto-starting next phase after {:?} completion. Next phase: {:?}",
            //     countdown_expired.phase,
            //     next_phase
            // );

            // self.timer_srv
            //     .update_timer(|t| {
            //         t.complete_phase(next_phase, &task.config.timer)?;
            //         Ok(())
            //     })
            //     .await?;

            self.timer_srv.load_state().await?;

            // let timer = self.timer_srv.get_current_timer().await;

            let timer_config = task.config.timer.clone();

            self.timer_srv
                .reset_timer_phase(timer_config.clone())
                .await
                .map_err(|e| domain::Error::RepositoryError {
                    message: format!("Failed to reset timer: {}", e),
                })?;

            // Start the timer tick loop for the new phase
            self.timer_srv
                .start_timer_tick_loop(Some(timer_config), None)
                .await
                .map_err(|e| domain::Error::RepositoryError {
                    message: format!("Failed to auto-start timer: {}", e),
                })?;

            // Get the current timer state AFTER all operations complete
            let current_timer = self.timer_srv.get_current_timer().await;

            log::info!(
                "Phase {:?}, Timer {:?}",
                current_timer.get_current_phase(),
                current_timer
            );

            log::info!(
                "About to emit STATUS_CHANGED with state: {:?}",
                current_timer.state()
            );

            self.emitter
                .emit(
                    domain::event_names::ui_listeners::timer::STATUS_CHANGED,
                    json!(current_timer.state()),
                )
                .map_err(|e| {
                    log::error!("Failed to emit STATUS_CHANGED: {}", e);
                    domain::Error::EventPublishingError {
                        message: format!(
                            "Failed to emit timer status changed event: {e}"
                        ),
                    }
                })?;

            log::info!("STATUS_CHANGED event emitted successfully");

            log::info!(
                "About to emit PHASE_COMPLETED with state: {:?}",
                current_timer.state()
            );

            self.emitter
                .emit(
                    domain::event_names::ui_listeners::timer::PHASE_COMPLETED,
                    json!(current_timer.state()),
                )
                .map_err(|e| {
                    log::error!("Failed to emit PHASE_COMPLETED: {}", e);
                    domain::Error::EventPublishingError {
                        message: format!(
                            "Failed to emit timer phase completed event: {e}"
                        ),
                    }
                })?;

            log::info!("PHASE_COMPLETED event emitted successfully");

            // Also emit task state to ensure UI has latest task info (sessions, etc.)
            self.emitter
                .emit(
                    domain::event_names::ui_listeners::task::PROGRESS_UPDATED,
                    json!(task),
                )
                .map_err(|e| {
                    log::error!("Failed to emit PROGRESS_UPDATED: {}", e);
                    domain::Error::EventPublishingError {
                        message: format!(
                            "Failed to emit task progress updated event: {e}"
                        ),
                    }
                })?;

            log::info!("PROGRESS_UPDATED event emitted successfully");

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
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "CountdownExpiredHandler"
    }
}
