use crate::adapters::events::app_emitter::Emitter;
use crate::adapters::EventHandler;
use async_trait::async_trait;
use domain::{Event, Result, config::events::ConfigReset};
use serde_json::json;
use std::any::TypeId;
use std::sync::Arc;

pub struct ConfigResetHandler {
    emitter: Arc<dyn Emitter>,
}

impl ConfigResetHandler {
    pub fn new(emitter: Arc<dyn Emitter>) -> Self {
        ConfigResetHandler { emitter }
    }
}

#[async_trait]
impl EventHandler for ConfigResetHandler {
    fn subscribes_to(&self) -> TypeId {
        TypeId::of::<ConfigReset>()
    }

    async fn handle(&self, event: Box<dyn Event>) -> Result<()> {
        let config_reset = event
            .as_any()
            .downcast_ref::<ConfigReset>()
            .ok_or(domain::Error::EventHandlingError {
                message: "Failed to downcast to ConfigReset".to_string(),
            })?;

        self.emitter
            .emit(
                domain::event_names::ui_listeners::config::CONFIG_RESET,
                json!(config_reset),
            )
            .map_err(|e| domain::Error::EventPublishingError {
                message: format!("Failed to emit config reset event: {e}"),
            })?;
        Ok(())
    }
}