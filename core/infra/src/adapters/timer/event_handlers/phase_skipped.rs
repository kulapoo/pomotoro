use async_trait::async_trait;
use domain::{Event, PhaseSkipped, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::TimerTickService;
use crate::adapters::events::EventHandler;
use crate::adapters::events::app_emitter::Emitter;

pub struct PhaseSkippedHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
}

impl PhaseSkippedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        timer_srv: Arc<TimerTickService>,
    ) -> Self {
        Self { emitter, timer_srv }
    }
}

#[async_trait]
impl EventHandler for PhaseSkippedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<PhaseSkipped>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let _phase_skipped = event
            .as_any()
            .downcast_ref::<domain::PhaseSkipped>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to skip phase".to_string(),
            })?;

        self.timer_srv.load_state().await?;
        let state_json = self.timer_srv.with_timer(|t| json!(t.state())).await;
        self.emitter
            .emit(
                domain::event_names::timer::PHASE_SKIPPED,
                state_json.clone(),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit phase skipped event: {e}"),
            })?;

        self.emitter
            .emit(domain::event_names::timer::PHASE_COMPLETED, state_json)
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit phase skipped event: {e}"),
            })?;

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
