use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result, TaskRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

/// UI-only emitter for the batch `TasksReset` event. Replaces the per-task
/// `TaskReset` UI events that the batch `reset_tasks` usecase used to emit,
/// so the React layer shows a single toast instead of N.
///
/// This handler MUST NOT touch the tick loop — the orchestrator that called
/// `reset_tasks` owns the `stop_timer_tick_loop` + `load_state` side effects.
pub struct TasksResetHandler {
    emitter: Arc<dyn Emitter>,
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
}

impl TasksResetHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        task_repo: Arc<dyn TaskRepository + Send + Sync>,
    ) -> Self {
        TasksResetHandler { emitter, task_repo }
    }
}

#[async_trait]
impl EventHandler for TasksResetHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::TasksReset>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let tasks_reset = event
            .as_any()
            .downcast_ref::<domain::TasksReset>()
            .ok_or(domain::Error::EventHandlingError {
            message: "Failed to handle batch task reset".to_string(),
        })?;

        let mut tasks = Vec::with_capacity(tasks_reset.task_ids.len());
        for &id in &tasks_reset.task_ids {
            match self.task_repo.get_by_id(id).await? {
                Some(task) => tasks.push(task),
                None => log::warn!(
                    "Task {} not found after batch reset; skipping",
                    id
                ),
            }
        }

        let payload = json!({
            "task_ids": tasks_reset.task_ids,
            "tasks": tasks,
            "version": tasks_reset.version,
            "occurred_at": tasks_reset.occurred_at,
        });

        self.emitter
            .emit(
                domain::event_names::ui_listeners::task::TASKS_RESET,
                payload.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit batch task reset event: {e}"),
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
