use async_trait::async_trait;
use domain::{
    BreakPhaseCompleted, ConfigRepository, Event, EventPublisher, Result,
    TaskActiveChanged, TaskRepository, TimerRepository,
    event_names::ui_listeners, task::CycleService,
};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::TimerTickService;
use crate::adapters::events::EventHandler;
use crate::adapters::events::app_emitter::Emitter;

use usecases::timer::{
    StartTimerPhaseCmd, pause_timer_phase, reset_timer_phase, start_timer_phase,
};
use usecases::{SwitchActiveTaskCmd, switch_active_task};

pub struct BreakPhaseCompletedHandler {
    emitter: Arc<dyn Emitter>,
    task_repository: Arc<dyn TaskRepository + Send + Sync>,
    timer_repository: Arc<dyn TimerRepository + Send + Sync>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    timer_srv: Arc<TimerTickService>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl BreakPhaseCompletedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        task_repository: Arc<dyn TaskRepository + Send + Sync>,
        timer_repository: Arc<dyn TimerRepository + Send + Sync>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
        timer_srv: Arc<TimerTickService>,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Self {
        Self {
            emitter,
            task_repository,
            timer_repository,
            config_repository,
            timer_srv,
            event_publisher,
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
        let config = self.config_repository.get_config().await?;

        // Only proceed if timer is running and task is completed
        if timer.is_running() && task.is_completed() {
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
                        "Break phase completed:: Next Task not found after task: {}",
                        current_task_id
                    ),
                })?;

                // Emit task active changed event
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

                // Stop the timer tick loop
                self.timer_srv.stop_timer_tick_loop().await.map_err(|e| {
                    domain::Error::EventHandlingError {
                        message: format!("Failed to stop timer tick loop: {e}"),
                    }
                })?;

                pause_timer_phase(
                    current_task_id,
                    self.task_repository.clone(),
                    self.timer_repository.clone(),
                    self.event_publisher.clone(),
                )
                .await?;

                switch_active_task(
                    self.task_repository.clone(),
                    self.timer_repository.clone(),
                    self.event_publisher.clone(),
                    SwitchActiveTaskCmd {
                        task_id: next_task.id(),
                    },
                )
                .await?;

                reset_timer_phase(
                    next_task.id(),
                    self.task_repository.clone(),
                    self.timer_repository.clone(),
                    self.event_publisher.clone(),
                )
                .await?;

                self.timer_srv.load_state().await?;

                if config.general.auto_start_work_after_break {
                    start_timer_phase(
                        self.task_repository.clone(),
                        self.timer_repository.clone(),
                        self.event_publisher.clone(),
                        StartTimerPhaseCmd {
                            task_id: Some(next_task.id()),
                        },
                    )
                    .await?;
                }
            }
        }

        Ok(())
    }
}
