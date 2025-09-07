use async_trait::async_trait;
use domain::{Event, PhaseCompleted, Result, event_names::ui_listeners};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::events::EventHandler;
use crate::adapters::events::app_emitter::Emitter;

pub struct PhaseCompletedHandler {
    emitter: Arc<dyn Emitter>,
}

impl PhaseCompletedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        Self { emitter }
    }
}

#[async_trait]
impl EventHandler for PhaseCompletedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<PhaseCompleted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(phase_completed) =
            event.as_any().downcast_ref::<PhaseCompleted>()
        {
            self.emitter
                .emit(
                    ui_listeners::timer::PHASE_COMPLETED,
                    json!(phase_completed.clone()),
                )
                .map_err(|e| domain::Error::RepositoryError {
                    message: format!(
                        "Failed to emit phase completed event: {e}"
                    ),
                })?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "PhaseCompletedHandler"
    }
}

impl From<PhaseCompletedHandler> for Box<dyn EventHandler> {
    fn from(handler: PhaseCompletedHandler) -> Self {
        Box::new(handler)
    }
}
