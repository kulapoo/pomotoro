use domain::EventPublisher;
use std::sync::Arc;
use tokio::sync::Mutex;
use usecases::get_config;

use crate::adapters::{
    InMemoryConfigRepository, InMemoryEventBus,
    events::{EventSubscriber, app_lifecycle, mem_event_bus::EventPublisherArc},
    task::{InMemoryTaskRepository, register_task_handlers},
    timer::event_handlers::register_timer_handlers,
};
use tauri::AppHandle;

use crate::adapters::{RodioAudioService, TimerService, audio::InMemoryAudioLibraryService};
use domain::timer::TimerService as DomainTimerService;

pub struct AppRegistry {
    pub task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
    pub config_repository: Arc<dyn domain::ConfigRepository + Send + Sync>,
    pub event_publisher: EventPublisherArc,
    pub timer_service: Arc<dyn DomainTimerService + Send + Sync>,
    pub audio_service: Arc<RodioAudioService>,
    #[allow(dead_code)]
    pub audio_library_service: Arc<Mutex<dyn usecases::audio::manage_library::AudioLibraryService>>,
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

pub fn register_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    app_handle: AppHandle,
) -> Result<(), BootstrapError> {
    let err_fn = |e: domain::Error| BootstrapError::EventSystem(e.to_string());
    register_timer_handlers(event_bus.clone(), app_handle.clone()).map_err(err_fn)?;
    register_task_handlers(event_bus, app_handle).map_err(err_fn)?;
    Ok(())
}

pub async fn bootstrap(app_handle: AppHandle) -> Result<AppRegistry, BootstrapError> {
    let config_repository: Arc<dyn domain::ConfigRepository + Send + Sync> =
        Arc::new(InMemoryConfigRepository::default());

    let config = get_config(&config_repository)
        .await
        .map_err(|e| BootstrapError::ConfigInit(e.to_string()))?;

    let task_defaults = config.task_defaults;
    let task_repository: Arc<dyn domain::TaskRepository + Send + Sync> =
        Arc::new(InMemoryTaskRepository::with_default_task(&task_defaults));

    let event_bus = Arc::new(InMemoryEventBus::new());
    let event_publisher: Arc<dyn EventPublisher + Send + Sync + 'static> = event_bus.clone();

    register_handlers(event_bus.clone(), app_handle.clone())?;

    let audio_service = Arc::new(RodioAudioService::new().map_err(|e| BootstrapError::AudioInit(e.to_string()))?);

    let audio_library_service: Arc<Mutex<dyn usecases::audio::manage_library::AudioLibraryService>> =
        Arc::new(Mutex::new(InMemoryAudioLibraryService::new()));

    let timer_service: Arc<dyn DomainTimerService + Send + Sync> = Arc::new(TimerService::new_with_services(
        event_publisher.clone(),
        Some(app_handle.clone()),
    ));

    let app_started =
        app_lifecycle::AppStarted::new(1, "v1.0.0".to_string(), true, true, true, Some(100), chrono::Utc::now());

    event_publisher.publish(Box::new(app_started));

    let ctx = AppRegistry {
        task_repository,
        config_repository,
        event_publisher,
        timer_service,
        audio_service,
        audio_library_service,
    };

    Ok(ctx)
}
