use async_trait::async_trait;
use domain::{
    BreakPhaseCompleted, ConfigRepository, Event, Result, TaskActiveChanged,
    TaskRepository, event_names::ui_listeners, task::CycleService,
};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::TimerTickService;
use crate::adapters::events::EventHandler;
use crate::adapters::events::app_emitter::Emitter;

pub struct BreakPhaseCompletedHandler {
    emitter: Arc<dyn Emitter>,
    task_repository: Arc<dyn TaskRepository + Send + Sync>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    timer_srv: Arc<TimerTickService>,
}

impl BreakPhaseCompletedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        task_repository: Arc<dyn TaskRepository + Send + Sync>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
        timer_srv: Arc<TimerTickService>,
    ) -> Self {
        Self {
            emitter,
            task_repository,
            config_repository,
            timer_srv,
        }
    }
}

#[async_trait]
impl EventHandler for BreakPhaseCompletedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<BreakPhaseCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let break_phase_completed = event
            .as_any()
            .downcast_ref::<BreakPhaseCompleted>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to complete break phase".to_string(),
            })?;

        // Emit break phase completed UI event
        self.emitter
            .emit(
                ui_listeners::timer::BREAK_PHASE_COMPLETED,
                json!(break_phase_completed.clone()),
            )
            .map_err(|e| domain::Error::RepositoryError {
                message: format!(
                    "Failed to emit break phase completed event: {e}"
                ),
            })?;

        let task = self
            .task_repository
            .get_by_id(break_phase_completed.task_id)
            .await?
            .ok_or(domain::Error::RepositoryError {
                message: format!(
                    "Break phase completed:: Task not found: {}",
                    break_phase_completed.task_id
                ),
            })?;

        self.timer_srv.load_state().await?;

        let timer = self.timer_srv.get_current_timer().await;

        if timer.is_running() && task.is_completed() {
            let config = self.config_repository.get_config().await?;
            let current_task_id = break_phase_completed.task_id;
            // Check if auto-cycling is enabled
            if CycleService::should_auto_cycle(&config.general) {
                let active_tasks =
                    self.task_repository.get_active_tasks().await?;

                let next_task = CycleService::select_next_task(
                    &active_tasks,
                    Some(&current_task_id),
                    &config.general.task_cycling_behavior,
                )
                .ok_or(domain::Error::RepositoryError {
                    message: format!(
                        "Break phase completed:: Next Task not found: {}",
                        current_task_id
                    ),
                })?;

                self.emitter
                    .emit(
                        ui_listeners::task::ACTIVE_CHANGED,
                        json!(TaskActiveChanged::new(
                            Some(current_task_id),
                            next_task.id(),
                            "Break phase: Task Active changed".to_string(),
                            1
                        )),
                    )
                    .map_err(|e| domain::Error::RepositoryError {
                        message: format!(
                            "Failed to emit active changed event: {e}"
                        ),
                    })?;
            }
        }

        Ok(())
    }
}
