use crate::adapters::EventHandler;
use crate::adapters::events::app_emitter::Emitter;
use async_trait::async_trait;
use domain::{Event, Result};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct AppStartedHandler {
    emitter: Arc<dyn Emitter>,
}

impl AppStartedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        AppStartedHandler { emitter }
    }
}

#[async_trait]
impl EventHandler for AppStartedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<domain::shared_kernel::events::AppStarted>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let app_started =
            event.as_any().downcast_ref::<domain::shared_kernel::events::AppStarted>();

        self.emitter
            .emit(domain::event_names::task::LIST_UPDATED, json!(app_started))
            .map_err(|e| domain::Error::RepositoryError {
                message: format!("Failed to emit app started event: {e}"),
            })?;
        Ok(())
    }
}
