use std::sync::Arc;

use crate::adapters::{task::InMemoryTaskRepository, InMemoryConfigRepository};
use tauri::AppHandle;
use usecases::HandlerRegistry;

use crate::adapters::{
    create_event_publisher_with_bus, DomainEventBus, EventPublisherArc,
    RodioAudioService, TimerService,
};

pub struct AppRegistry {

    pub task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
    pub config_repository: Arc<dyn domain::ConfigRepository + Send + Sync>,

    #[allow(dead_code)]
    pub event_handlers: Arc<HandlerRegistry>,

    pub event_bus: Arc<DomainEventBus>,
    pub event_publisher: EventPublisherArc,

    pub timer_service: Arc<dyn domain::timer::TimerService + Send + Sync>,
    pub audio_service: Arc<RodioAudioService>,
}

fn init_events(
    _event_handlers: &Arc<HandlerRegistry>
) -> Result<(), BootstrapError> {



    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum BootstrapError {
    #[error("Failed to initialize config repository: {0}")]
    ConfigInit(String),
    #[error("Failed to initialize audio service: {0}")]
    AudioInit(String),
    #[error("Failed to create event system: {0}")]
    EventSystem(String),

    #[error("Failed to orchestrate event system: {0}")]
    OrchestrationError(String),
}

impl From<BootstrapError> for String {
    fn from(err: BootstrapError) -> Self {
        err.to_string()
    }
}

pub async fn bootstrap(app_handle: AppHandle) -> Result<AppRegistry, BootstrapError> {
    let task_repository: Arc<dyn domain::TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::with_default_task());
    let config_repository: Arc<dyn domain::ConfigRepository + Send + Sync> = Arc::new(InMemoryConfigRepository::default());

    let (event_publisher, event_bus): (EventPublisherArc, Arc<DomainEventBus>) =
        create_event_publisher_with_bus(app_handle.clone());

    let audio_service = Arc::new(
        RodioAudioService::new().map_err(|e| BootstrapError::AudioInit(e.to_string()))?
    );

    let timer_service: Arc<dyn domain::timer::TimerService + Send + Sync> =
        Arc::new(TimerService::new_with_services(event_publisher.clone(), Some(app_handle.clone())));

    let event_handlers = Arc::new(HandlerRegistry::new());

    usecases::bootstrap(&task_repository, &config_repository, &timer_service, &event_publisher).await.map_err(|e| BootstrapError::OrchestrationError(e.to_string()))?;

    init_events(&event_handlers).map_err(|e| BootstrapError::EventSystem(e.to_string()))?;

    let ctx = AppRegistry {
        event_handlers,
        task_repository,
        config_repository,
        event_bus,
        event_publisher,
        timer_service,
        audio_service,
    };

    Ok(ctx)
}
