use std::sync::Arc;

use crate::core::mocks::ui::app_handle::MockAppHandle;
use anyhow::Result;
use domain::{ConfigRepository, TaskRepository};
use infra::adapters::TimerTickService;
use infra::adapters::events::EventSubscriber;
use infra::adapters::events::app_emitter::Emitter;
use infra::adapters::events::app_started_handler::AppStartedHandler;
use infra::adapters::task::event_handlers::register_task_handlers;
use infra::adapters::timer::event_handlers::register_timer_handlers;

pub fn register_test_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    app_handle: MockAppHandle,
    task_repository: Arc<dyn TaskRepository>,
    timer_tick_service: Arc<TimerTickService>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
) -> Result<()> {
    let emitter: Arc<dyn Emitter> = Arc::new(app_handle);

    event_bus.subscribe(Box::new(AppStartedHandler::new(emitter.clone())))?;
    register_task_handlers(event_bus.clone(), emitter.clone())?;
    register_timer_handlers(
        event_bus.clone(),
        emitter.clone(),
        timer_tick_service.clone(),
        task_repository.clone(),
        config_repository.clone(),
    )?;

    // Note: We're not registering notification and audio handlers in tests
    // as they don't emit events and are tested separately

    Ok(())
}
