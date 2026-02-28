use std::sync::Arc;

use crate::adapters::events::{EventSubscriber, app_emitter::Emitter};
use anyhow::Result;

use super::{ConfigResetHandler, ConfigUpdatedHandler};

pub fn register_config_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    emitter: Arc<dyn Emitter>,
) -> Result<()> {
    // Register ConfigUpdated handler
    event_bus
        .subscribe(Box::new(ConfigUpdatedHandler::new(emitter.clone())))?;

    // Register ConfigReset handler
    event_bus.subscribe(Box::new(ConfigResetHandler::new(emitter.clone())))?;

    Ok(())
}
