use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result, TaskRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

/// UI-only emitter for `TaskReset`.
///
/// Per the tick-loop ownership contract, this handler MUST NOT stop the tick
/// loop. The orchestrator that called `reset_task` owns the
/// `stop_timer_tick_loop` + `load_state` side effects.
pub struct TaskResetHandler {
    emitter: Arc<dyn Emitter>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
}

impl TaskResetHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        task_repo: Arc<dyn TaskRepository + Send + Sync>,
    ) -> Self {
        TaskResetHandler { emitter, task_repo }
    }
}

#[async_trait]
impl EventHandler for TaskResetHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TaskReset>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let task_reset = event
            .as_any()
            .downcast_ref::<domain::TaskReset>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to reset task".to_string(),
            })?;

        let task = self.task_repo.get_by_id(task_reset.task_id).await?;

        let Some(task) = task else {
            log::warn!(
                "Task {} not found after reset; skipping emit",
                task_reset.task_id
            );
            return Ok(());
        };

        let payload = json!({
            "task_id": task_reset.task_id,
            "name": task_reset.name,
            "description": task_reset.description,
            "max_sessions": task_reset.max_sessions,
            "tags": task_reset.tags,
            "version": task_reset.version,
            "occurred_at": task_reset.occurred_at,
            "task": task,
        });

        self.emitter
            .emit(
                domain::event_names::ui_listeners::task::TASK_RESET,
                payload.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task reset event: {e}"),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::task::LIST_UPDATED,
                payload,
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit task list updated event: {e}"),
            })?;
        Ok(())
    }
}
