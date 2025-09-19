use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::EventHandler;
use async_trait::async_trait;
use domain::{Event, Result, config::events::ConfigUpdated};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct ConfigUpdatedHandler {
    emitter: Arc<dyn Emitter>,
}

impl ConfigUpdatedHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        ConfigUpdatedHandler { emitter }
    }
}

#[async_trait]
impl EventHandler for ConfigUpdatedHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<ConfigUpdated>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let config_updated = event
            .as_any()
            .downcast_ref::<ConfigUpdated>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to downcast to ConfigUpdated".to_string(),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::config::CONFIG_UPDATED,
                json!(config_updated),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit config updated event: {e}"),
            })?;
        Ok(())
    }
}