use std::sync::Arc;

use crate::core::mocks::ui::app_handle::MockAppHandle;
use anyhow::Result;
use domain::{ConfigRepository, EventPublisher, TaskCyclerService, TaskRepository, TimerRepository};
use infra::adapters::TimerTickService;
use infra::adapters::events::EventSubscriber;
use infra::adapters::events::app_emitter::Emitter;
use infra::adapters::events::app_started_handler::AppStartedHandler;
use infra::adapters::config::register_config_handlers;
use infra::adapters::task::event_handlers::register_task_handlers;
use infra::adapters::timer::event_handlers::register_timer_handlers;

pub fn register_test_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    app_handle: MockAppHandle,
    task_repository: Arc<dyn TaskRepository>,
    timer_tick_service: Arc<TimerTickService>,
    config_repository: Arc<dyn ConfigRepository + Send + Sync>,
    task_cycling_service: Arc<dyn TaskCyclerService + Send + Sync>,
    timer_repository: Arc<dyn TimerRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
) -> Result<()> {
    let emitter: Arc<dyn Emitter> = Arc::new(app_handle);

    event_bus.subscribe(Box::new(AppStartedHandler::new(emitter.clone())))?;
    register_task_handlers(
        event_bus.clone(),
        emitter.clone(),
        task_repository.clone(),
        task_cycling_service,
        timer_repository,
        event_publisher,
        timer_tick_service.clone(),
    )?;
    register_timer_handlers(
        event_bus.clone(),
        emitter.clone(),
        timer_tick_service.clone(),
        task_repository.clone(),
        config_repository.clone(),
    )?;
    register_config_handlers(event_bus.clone(), emitter.clone())?;

    // Note: We're not registering notification and audio handlers in tests
    // as they don't emit events and are tested separately

    Ok(())
}
