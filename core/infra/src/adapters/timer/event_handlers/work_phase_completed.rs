use async_trait::async_trait;
use domain::{Event, Result, WorkPhaseCompleted, event_names::ui_listeners};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::events::EventHandler;
use crate::adapters::events::app_emitter::Emitter;

/// Emits the `WORK_PHASE_COMPLETED` UI event when a work phase completes.
///
/// Task-completion (`TaskCompleted`) is intentionally NOT emitted here.
/// The fully-done moment is owned by the `complete_timer_phase` /
/// `skip_timer_phase` use cases, which finalize the task only after the
/// trailing break has been taken (or skipped) — matching canonical
/// Pomodoro semantics.
pub struct WorkPhaseCompletedHandler {
    emitter: Arc<dyn Emitter>,
}

impl WorkPhaseCompletedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        Self { emitter }
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

        Ok(())
    }
}
