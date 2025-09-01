use std::sync::Arc;

use infra::adapters::events::app_started_handler::AppStartedHandler;
use infra::adapters::events::EventSubscriber;
use infra::adapters::events::app_emitter::Emitter;
use infra::adapters::timer::event_handlers::register_timer_handlers;
use infra::adapters::task::event_handlers::register_task_handlers;
use crate::core::mocks::ui::app_handle::MockAppHandle;
use anyhow::Result;

pub fn register_test_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    app_handle: MockAppHandle,
) -> Result<()> {
    let emitter: Arc<dyn Emitter> = Arc::new(app_handle);

    event_bus
        .subscribe(Box::new(AppStartedHandler::new(emitter.clone())))?;
    register_task_handlers(event_bus.clone(), emitter.clone())?;
    register_timer_handlers(event_bus.clone(), emitter)?;

    // Note: We're not registering notification and audio handlers in tests
    // as they don't emit events and are tested separately

    Ok(())
}