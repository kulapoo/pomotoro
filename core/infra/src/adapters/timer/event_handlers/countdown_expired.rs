use crate::adapters::{
    TimerTickService,
    events::{EventHandler, app_emitter::Emitter},
};
use async_trait::async_trait;
use domain::{
    ConfigRepository, Event, EventPublisher, Phase, Result, Task, TaskId,
    TaskRepository, TimerRepository, event_names::ui_listeners,
    timer::events::CountdownExpired,
};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;
use usecases::timer::{ProgressPhaseCmd, progress_phase};

/// Event handler that triggers phase progression when countdown naturally
/// expires.
///
/// Delegates all domain orchestration (phase completion, auto-start, task
/// cycling) to the `progress_phase` usecase. This handler only performs
/// infrastructure concerns: starting/stopping the tick loop and emitting
/// UI events based on the outcome.
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

    /// Shared tail of the `Started` and `Paused` outcomes: emit
    /// `task:auto_advanced` (when the cycle moved to a new task),
    /// `task:progress_updated`, and `screen_blocker:activate` (when a
    /// long-break block message applies). `timer_json` is supplied by the
    /// caller since each arm sources it differently.
    async fn emit_post_phase_events(
        &self,
        task: &Task,
        cycled_to: Option<TaskId>,
        from_task_id: TaskId,
        timer_json: serde_json::Value,
        block_message: Option<&str>,
    ) -> Result<()> {
        if let Some(to_task_id) = cycled_to {
            self.emitter
                .emit(
                    ui_listeners::task::AUTO_ADVANCED,
                    json!({
                        "from_task_id": from_task_id,
                        "to_task_id": to_task_id,
                        "to_task": task,
                        "timer": timer_json,
                    }),
                )
                .map_err(|e| domain::Error::EventPublishingError {
                    message: format!("Failed to emit auto-advanced event: {e}"),
                })?;
        }

        self.emitter
            .emit(ui_listeners::task::PROGRESS_UPDATED, json!(task))
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!(
                    "Failed to emit task progress updated event: {e}"
                ),
            })?;

        if let Some(message) = block_message {
            self.emitter
                .emit(
                    ui_listeners::screen_blocker::ACTIVATE,
                    json!({ "message": message }),
                )
                .map_err(|e| domain::Error::EventPublishingError {
                    message: format!(
                        "Failed to emit screen_blocker activate event: {e}"
                    ),
                })?;
        }

        Ok(())
    }
}

#[async_trait]
impl EventHandler for CountdownExpiredHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<CountdownExpired>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let countdown_expired = event
            .as_any()
            .downcast_ref::<CountdownExpired>()
            .ok_or_else(|| domain::Error::RepositoryError {
                message: "Failed to downcast to CountdownExpired event"
                    .to_string(),
            })?;

        // Acquire the orchestration lock so we serialize against any
        // concurrent manual command (skip / pause / etc.) or tray handler.
        // Without this, a natural expiry racing with a manual skip can
        // collide on the DB and tick loop.
        let _orchestration_lock = self.timer_srv.orchestration_lock().await;

        let outcome = progress_phase(
            self.task_repository.clone(),
            self.timer_repository.clone(),
            self.config_repository.clone(),
            self.event_publisher.clone(),
            ProgressPhaseCmd {
                task_id: countdown_expired.task_id,
                from_phase: countdown_expired.phase,
            },
        )
        .await?;

        match outcome {
            usecases::timer::PhaseOutcome::Started {
                task,
                next_phase,
                cycled_to,
                block_message,
                ..
            } => {
                self.timer_srv.load_state().await?;

                if !(task.is_completed() && next_phase == Phase::Work) {
                    let timer_config = task.config().timer.clone();
                    self.timer_srv
                        .start_timer_tick_loop(Some(timer_config))
                        .await
                        .map_err(|e| domain::Error::RepositoryError {
                            message: format!(
                                "Failed to auto-start timer: {}",
                                e
                            ),
                        })?;
                }

                let state_json =
                    self.timer_srv.with_timer(|t| json!(t.state())).await;

                self.emitter
                    .emit(
                        ui_listeners::timer::PHASE_COMPLETED,
                        json!({ "timer": state_json, "task": task }),
                    )
                    .map_err(|e| domain::Error::EventPublishingError {
                        message: format!(
                            "Failed to emit timer phase completed event: {e}"
                        ),
                    })?;

                let timer_json = self.timer_srv.with_timer(|t| json!(t)).await;
                self.emit_post_phase_events(
                    &task,
                    cycled_to,
                    countdown_expired.task_id,
                    timer_json,
                    block_message.as_deref(),
                )
                .await?;
            }

            usecases::timer::PhaseOutcome::Paused {
                task,
                timer,
                cycled_to,
                block_message,
                ..
            } => {
                self.timer_srv.load_state().await?;

                self.emitter
                    .emit(
                        ui_listeners::timer::STATUS_CHANGED,
                        json!(timer.state()),
                    )
                    .map_err(|e| domain::Error::RepositoryError {
                        message: format!(
                            "Failed to emit timer status changed event: {e}"
                        ),
                    })?;

                self.emit_post_phase_events(
                    &task,
                    cycled_to,
                    countdown_expired.task_id,
                    json!(timer),
                    block_message.as_deref(),
                )
                .await?;
            }

            usecases::timer::PhaseOutcome::Stopped { .. } => {
                self.timer_srv.stop_timer_tick_loop().await.map_err(|e| {
                    domain::Error::RepositoryError {
                        message: format!("Failed to stop timer tick loop: {e}"),
                    }
                })?;

                self.timer_srv.load_state().await?;
                let state_json =
                    self.timer_srv.with_timer(|t| json!(t.state())).await;

                self.emitter
                    .emit(ui_listeners::timer::STATUS_CHANGED, state_json)
                    .map_err(|e| domain::Error::EventPublishingError {
                        message: format!(
                            "Failed to emit timer status changed event: {e}"
                        ),
                    })?;
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "CountdownExpiredHandler"
    }
}
