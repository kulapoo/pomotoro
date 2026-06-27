use async_trait::async_trait;
use domain::{Event, PhaseSkipped, Result, TaskRepository};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

use crate::adapters::TimerTickService;
use crate::adapters::events::EventHandler;
use crate::adapters::events::app_emitter::Emitter;

pub struct PhaseSkippedHandler {
    emitter: Arc<dyn Emitter>,
    timer_srv: Arc<TimerTickService>,
    task_repository: Arc<dyn TaskRepository + Send + Sync>,
}

impl PhaseSkippedHandler {
    pub fn new(
        emitter: Arc<dyn Emitter>,
        timer_srv: Arc<TimerTickService>,
        task_repository: Arc<dyn TaskRepository + Send + Sync>,
    ) -> Self {
        Self {
            emitter,
            timer_srv,
            task_repository,
        }
    }
}

#[async_trait]
impl EventHandler for PhaseSkippedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<PhaseSkipped>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let phase_skipped = event
            .as_any()
            .downcast_ref::<domain::PhaseSkipped>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to skip phase".to_string(),
            })?;

        self.timer_srv.load_state().await?;
        let state_json = self.timer_srv.with_timer(|t| json!(t.state())).await;

        // Embed the bound Task so the payload shape matches the
        // natural-expiry path in CountdownExpiredHandler. The task itself
        // is unchanged by a skip; this is purely for shape consistency.
        let task_json = match self
            .task_repository
            .get_by_id(phase_skipped.task_id)
            .await
        {
            Ok(Some(task)) => json!(task),
            Ok(None) => {
                log::warn!(
                    "PhaseSkippedHandler: task {} not found; emitting task: null",
                    phase_skipped.task_id
                );
                json!(null)
            }
            Err(e) => {
                log::warn!(
                    "PhaseSkippedHandler: failed to load task {}: {e}; emitting task: null",
                    phase_skipped.task_id
                );
                json!(null)
            }
        };

        let payload = json!({ "timer": state_json, "task": task_json });

        self.emitter
            .emit(domain::event_names::timer::PHASE_SKIPPED, payload.clone())
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit phase skipped event: {e}"),
            })?;

        self.emitter
            .emit(domain::event_names::timer::PHASE_COMPLETED, payload)
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
