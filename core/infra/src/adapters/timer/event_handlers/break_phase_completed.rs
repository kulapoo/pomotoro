use async_trait::async_trait;
use domain::{BreakPhaseCompleted, Event, Result, event_names::ui_listeners};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::events::EventHandler;
use crate::adapters::events::app_emitter::Emitter;

/// Emits the `BREAK_PHASE_COMPLETED` UI event when a break phase completes.
///
/// Task auto-cycling logic has been moved to the `progress_phase` usecase
/// (invoked by `CountdownExpiredHandler`) to eliminate the race condition
/// where two handlers concurrently mutated the singleton timer.
pub struct BreakPhaseCompletedHandler {
    emitter: Arc<dyn Emitter>,
}

impl BreakPhaseCompletedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        Self { emitter }
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

        Ok(())
    }
}
