use async_trait::async_trait;
use domain::{
    Event, Result, TaskCompleted, TaskRepository, WorkPhaseCompleted,
    event_names::ui_listeners,
};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::events::EventHandler;
use crate::adapters::events::app_emitter::Emitter;

/// Emits the `WORK_PHASE_COMPLETED` UI event when a work phase completes.
///
/// Also emits `task:task_completed` when the task has actually transitioned
/// to completed (checked via `task.is_completed()` rather than the old
/// `remaining_sessions <= 1` heuristic which could false-fire at task
/// creation when `max_sessions == 1`).
pub struct WorkPhaseCompletedHandler {
    emitter: Arc<dyn Emitter>,
    task_repository: Arc<dyn TaskRepository + Send + Sync>,
}

impl WorkPhaseCompletedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        task_repository: Arc<dyn TaskRepository + Send + Sync>,
    ) -> Self {
        Self {
            emitter,
            task_repository,
        }
    }
}

#[async_trait]
impl EventHandler for WorkPhaseCompletedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<WorkPhaseCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let work_phase_completed = event
            .as_any()
            .downcast_ref::<WorkPhaseCompleted>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to complete work phase".to_string(),
            })?;

        self.emitter
            .emit(
                ui_listeners::timer::WORK_PHASE_COMPLETED,
                json!(work_phase_completed.clone()),
            )
            .map_err(|e| domain::Error::RepositoryError {
                message: format!(
                    "Failed to emit work phase completed event: {e}"
                ),
            })?;

        let task = self
            .task_repository
            .get_by_id(work_phase_completed.task_id)
            .await?
            .ok_or(domain::Error::RepositoryError {
                message: format!(
                    "Work phase completed:: Task not found: {}",
                    work_phase_completed.task_id
                ),
            })?;

        if task.is_completed() {
            let task_completed = TaskCompleted::new(
                work_phase_completed.task_id,
                task.max_sessions(),
                1,
            );
            self.emitter
                .emit(ui_listeners::task::TASK_COMPLETED, json!(task_completed))
                .map_err(|e| domain::Error::RepositoryError {
                    message: format!(
                        "Failed to emit task completed event: {e}"
                    ),
                })?;
        }

        Ok(())
    }
}
