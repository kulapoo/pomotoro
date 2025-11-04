use crate::adapters::{
    TimerTickService,
    events::{EventHandler, app_emitter::Emitter},
};
use async_trait::async_trait;
use domain::{
    ConfigRepository, Phase, StateTransitions, timer::events::CountdownExpired,
};
use domain::{Error, EventPublisher, TaskRepository, TimerRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

/// Event handler that triggers phase completion when countdown naturally expires
pub struct CountdownExpiredHandler {
    emitter: Arc<dyn Emitter>,
}

impl CountdownExpiredHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        Self { emitter }
    }
}

#[async_trait]
impl EventHandler for CountdownExpiredHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<CountdownExpired>()
    }

    async fn handle(&self, event: Box<dyn domain::Event>) -> Result<(), Error> {
        let _countdown_expired = event
            .as_any()
            .downcast_ref::<CountdownExpired>()
            .ok_or_else(|| Error::RepositoryError {
                message: "Failed to downcast to CountdownExpired event"
                    .to_string(),
            })?;

        // let config = self.config_repository.get_config().await?;

        // let task = self
        //     .task_repository
        //     .get_by_id(countdown_expired.task_id)
        //     .await?
        //     .ok_or(domain::Error::RepositoryError {
        //         message: format!(
        //             "Countdown expired:: Task not found: {}",
        //             countdown_expired.task_id
        //         ),
        //     })?;

        // let (should_auto_start, next_phase) =
        //     match countdown_expired.phase.clone() {
        //         Phase::Work => {
        //             // Work phase expired, check if we should auto-start break
        //             (
        //                 config.general.auto_start_breaks,
        //                 Phase::determine_next_break_type(
        //                     task.current_sessions,
        //                     task.config.timer.sessions_until_long_break,
        //                 ),
        //             )
        //         }
        //         Phase::ShortBreak | Phase::LongBreak => {
        //             // Break phase expired, check if we should auto-start work
        //             (config.general.auto_start_work_after_break, Phase::Work)
        //         }
        //     };

        // if should_auto_start {
        //     self.timer_srv.load_state().await?;

        //     let timer = self.timer_srv.get_current_timer().await;
        //     let _ = timer
        //         .clone()
        //         .complete_phase(next_phase, &task.config.timer)?;

        //     self.timer_srv
        //         .update_timer(|t| {
        //             *t = timer.clone();
        //             Ok(())
        //         })
        //         .await?;

        //     log::info!(
        //         "COUNTDOWN PHASE: phase - {:?}, next phase - {:?}, timer state - {:?}",
        //         countdown_expired.phase,
        //         next_phase,
        //         timer.state()
        //     );

        //     let _ = self
        //         .timer_srv
        //         .start_timer_tick_loop(Some(task.config.timer.clone()), None)
        //         .await
        //         .map_err(|e| domain::Error::RepositoryError { message: e })?;

        //     self.emitter
        //         .emit(
        //             domain::event_names::ui_listeners::timer::STATUS_CHANGED,
        //             json!(timer.state()),
        //         )
        //         .map_err(|e| domain::Error::EventPublishingError {
        //             message: format!("Failed to emit timer started event: {e}"),
        //         })?;
        // }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "CountdownExpiredHandler"
    }
}
