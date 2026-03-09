use crate::adapters::EventHandler;
use crate::adapters::TimerTickService;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result, TaskRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct TaskCompletedHandler {
    emitter: Arc<dyn Emitter>,
    task_repository: Arc<dyn TaskRepository + Send + Sync>,
    timer_srv: Arc<TimerTickService>,
}

impl TaskCompletedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        task_repository: Arc<dyn TaskRepository + Send + Sync>,
        timer_srv: Arc<TimerTickService>,
    ) -> Self {
        TaskCompletedHandler {
            emitter,
            task_repository,
            timer_srv,
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
                message: "Failed to complete task".to_string(),
            })?;
        let task = self
            .task_repository
            .get_by_id(task_completed.task_id)
            .await?
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to complete task".to_string(),
            })?;

        let timer_config = task.get_config().timer.clone();

        self.timer_srv.stop_timer_tick_loop().await.map_err(|e| {
            domain::Error::EventHandlingError {
                message: format!("Failed to stop timer tick loop: {e}"),
            }
        })?;

        self.timer_srv.reset_timer(timer_config.clone()).await?;
        self.timer_srv.load_state().await?;

        let state_json = self.timer_srv.with_timer(|t| json!(t.state())).await;

        self.emitter
            .emit(
                domain::event_names::task::LIST_UPDATED,
                json!(task_completed),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task completed event: {e}"),
            })?;

        self.emitter
            .emit(domain::event_names::timer::STATUS_CHANGED, state_json)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task completed event: {e}"),
            })?;

        Ok(())
    }
}
