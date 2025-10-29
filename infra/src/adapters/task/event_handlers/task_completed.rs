use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::TimerTickService;
use async_trait::async_trait;
use domain::{
    ConfigRepository, Event, Result, TaskCyclerService, TaskRepository,
    task::services::AutoCycleService,
};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskCompletedHandler {
    emitter: Arc<dyn Emitter>,
    task_repository: Arc<dyn TaskRepository + Send + Sync>,
    timer_srv: Arc<TimerTickService>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    cycling_service: Arc<dyn TaskCyclerService + Send + Sync>,
}

impl TaskCompletedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        task_repository: Arc<dyn TaskRepository + Send + Sync>,
        timer_srv: Arc<TimerTickService>,
        config_repository: Arc<dyn ConfigRepository + Send + Sync>,
        cycling_service: Arc<dyn TaskCyclerService + Send + Sync>,
    ) -> Self {
        TaskCompletedHandler {
            emitter,
            task_repository,
            timer_srv,
            config_repository,
            cycling_service,
        }
    }
}

#[async_trait]
impl EventHandler for TaskCompletedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_completed = event
            .as_any()
            .downcast_ref::<domain::TaskCompleted>()
            .ok_or(domain::Error::EventHandlingError {
            message: format!("Failed to complete task"),
        })?;
        let task = self.task_repository.get_by_id(task_completed.task_id.clone()).await?.ok_or(
            domain::Error::EventHandlingError {
                message: format!("Failed to complete task"),
            }
        )?;
        let timer_config = task.get_config().timer.clone();

        self.timer_srv.load_state().await?;
        self.timer_srv
            .stop_timer_tick_loop()
            .await
            .map_err(|e| domain::Error::EventHandlingError {
                message: format!("Failed to stop timer tick loop: {e}"),
            })?;

        self.timer_srv.reset_timer(timer_config.clone()).await?;

        let timer = self.timer_srv.get_current_timer().await;

        // Check if AutoCycle is enabled and trigger cycling
        let config = self.config_repository.get_config().await?;
        if AutoCycleService::should_auto_cycle(&config.general) {
            // Get available tasks for cycling
            let available_tasks = self.task_repository.get_active_tasks().await?;

            // Find next task using pure domain logic
            if let Some(next_task) = AutoCycleService::select_next_task(
                &available_tasks,
                Some(&task_completed.task_id),
                &config.general.task_cycling_behavior,
            ) {
                // Use existing cycling infrastructure to switch to next task
                // This will handle all the timer updates and UI notifications
                let _cycle_result = self.cycling_service
                    .cycle_to_next_active_task(Some(task_completed.task_id.clone()))
                    .await?;

                // Log the auto-cycle action for debugging
                tracing::info!(
                    "AutoCycle: Switched from task {} to task {}",
                    task_completed.task_id,
                    next_task.id
                );
            } else {
                tracing::debug!("AutoCycle: No eligible tasks found for cycling");
            }
        }

        self.emitter
            .emit(
                domain::event_names::timer::PHASE_COMPLETED,
                json!(timer.state()),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit timer reset event: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::task::LIST_UPDATED,
                json!(task_completed),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task completed event: {e}"),
            })?;

        Ok(())
    }
}
