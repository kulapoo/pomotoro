use async_trait::async_trait;
use domain::{Event, PhaseSkipped, Result, event_names::ui_listeners};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::events::EventHandler;
use crate::adapters::events::app_emitter::Emitter;

pub struct PhaseSkippedHandler {
    emitter: Arc<dyn Emitter>,
}

impl PhaseSkippedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        Self { emitter }
    }
}

#[async_trait]
impl EventHandler for PhaseSkippedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<PhaseSkipped>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        if let Some(phase_skipped) =
            event.as_any().downcast_ref::<PhaseSkipped>()
        {
            self.emitter
                .emit(
                    ui_listeners::timer::PHASE_SKIPPED,
                    json!(phase_skipped.clone()),
                )
                .map_err(|e| domain::Error::RepositoryError {
                    message: format!("Failed to emit phase skipped event: {e}"),
                })?;
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "PhaseSkippedHandler"
    }
}

impl From<PhaseSkippedHandler> for Box<dyn EventHandler> {
    fn from(handler: PhaseSkippedHandler) -> Self {
        Box::new(handler)
    }
}
