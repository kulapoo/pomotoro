use crate::adapters::{
    TimerTickService,
    events::{EventHandler, app_emitter::Emitter},
};
use async_trait::async_trait;
use domain::{
    ConfigRepository, Phase, TimerRepository, timer::events::CountdownExpired,
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
}

impl CountdownExpiredHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        timer_srv: Arc<TimerTickService>,
        task_repository: Arc<dyn TaskRepository + Send + Sync>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
        timer_repository: Arc<dyn TimerRepository + Send + Sync>,
    ) -> Self {
        Self {
            emitter,
            timer_srv,
            task_repository,
            config_repository,
            timer_repository,
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
        let task = self
            .task_repository
            .get_by_id(countdown_expired.task_id)
            .await?
            .ok_or(domain::Error::RepositoryError {
                message: format!(
                    "Countdown expired: Task not found: {}",
                    countdown_expired.task_id
                ),
            })?;

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
            // complete_timer_phase(
            //     task_id,
            //     self.task_repository.clone(),
            //     self.timer_repository.clone(),
            //     self.config_repository.clone(),
            // )
            // .await?;
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

            // self.timer_srv.load_state().await?;

            // let timer = self.timer_srv.get_current_timer().await;

            // let timer_config = task.config.timer.clone();

            // self.timer_srv
            //     .reset_timer_phase(timer_config.clone())
            //     .await
            //     .map_err(|e| domain::Error::RepositoryError {
            //         message: format!("Failed to reset timer: {}", e),
            //     })?;

            // // Start the timer tick loop for the new phase
            // self.timer_srv
            //     .start_timer_tick_loop(Some(timer_config), None)
            //     .await
            //     .map_err(|e| domain::Error::RepositoryError {
            //         message: format!("Failed to auto-start timer: {}", e),
            //     })?;

            // // Load and emit the current timer state to update UI

            // log::info!(
            //     "Phase {:?}, Timer {:?}",
            //     timer.get_current_phase(),
            //     timer
            // );

            // self.emitter
            //     .emit(
            //         domain::event_names::ui_listeners::timer::STATUS_CHANGED,
            //         json!(timer.state()),
            //     )
            //     .map_err(|e| domain::Error::EventPublishingError {
            //         message: format!(
            //             "Failed to emit timer status changed event: {e}"
            //         ),
            //     })?;

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
