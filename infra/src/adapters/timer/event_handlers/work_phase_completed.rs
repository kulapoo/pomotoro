use async_trait::async_trait;
use domain::{Event, Result, event_names::ui_listeners, WorkPhaseCompleted};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::events::EventHandler;
use crate::adapters::events::app_emitter::Emitter;

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
                message: format!("Failed to complete work phase"),
            })?;

        self.emitter
            .emit(
                ui_listeners::timer::WORK_PHASE_COMPLETED,
                json!(work_phase_completed.clone()),
            )
            .map_err(|e| domain::Error::RepositoryError {
                message: format!("Failed to emit work phase completed event: {e}"),
            })?;

        Ok(())
    }
}
