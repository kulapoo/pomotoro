use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result, TaskRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

/// Emits the `task:list_updated` and `task:task_completed` UI events when a
/// task is completed, embedding the full `Task` object so the React
/// EventBus can direct-map the payload into the `activeTask` store slice
/// without an IPC round-trip.
///
/// This handler is a pure UI side-effect translator. It intentionally does
/// NOT stop or reset the timer — that orchestration belongs in the use case
/// / command layer that has full context (manual complete vs. auto-cycle).
/// Resetting the timer here would race with the cycling logic in
/// `progress_phase`, which emits `TaskCompleted` at the trailing-break
/// boundary while the timer is mid-transition.
pub struct TaskCompletedHandler {
    emitter: Arc<dyn Emitter>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
}

impl TaskCompletedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        task_repo: Arc<dyn TaskRepository + Send + Sync>,
    ) -> Self {
        TaskCompletedHandler { emitter, task_repo }
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

        let task = self.task_repo.get_by_id(task_completed.task_id).await?;

        let Some(task) = task else {
            log::warn!(
                "Task {} not found after completion; skipping emit",
                task_completed.task_id
            );
            return Ok(());
        };

        let payload = json!({
            "task_id": task_completed.task_id,
            "total_sessions": task_completed.total_sessions,
            "completed_at": task_completed.completed_at,
            "version": task_completed.version,
            "occurred_at": task_completed.occurred_at,
            "task": task,
        });

        self.emitter
            .emit(domain::event_names::task::LIST_UPDATED, payload.clone())
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task completed event: {e}"),
            })?;

        self.emitter
            .emit(domain::event_names::task::TASK_COMPLETED, payload)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task completed event: {e}"),
            })?;

        Ok(())
    }
}
