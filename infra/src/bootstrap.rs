use std::sync::Arc;
use domain::EventPublisher;
use tokio::sync::Mutex;

use crate::adapters::{events::{mem_event_bus::EventPublisherArc, app_lifecycle}, task::InMemoryTaskRepository, InMemoryEventBus, InMemoryConfigRepository};
use tauri::AppHandle;

use crate::adapters::{
    RodioAudioService, TimerService,
    audio::InMemoryAudioLibraryService
};

pub struct AppRegistry {
    pub task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
    pub config_repository: Arc<dyn domain::ConfigRepository + Send + Sync>,
    pub event_publisher: EventPublisherArc,
    pub timer_service: Arc<dyn domain::timer::TimerService + Send + Sync>,
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

pub async fn bootstrap(app_handle: AppHandle) -> Result<AppRegistry, BootstrapError> {
    let task_repository: Arc<dyn domain::TaskRepository + Send + Sync> = Arc::new(InMemoryTaskRepository::with_default_task());
    let config_repository: Arc<dyn domain::ConfigRepository + Send + Sync> = Arc::new(InMemoryConfigRepository::default());

    let event_publisher: Arc<dyn EventPublisher + Send + Sync + 'static> = Arc::new(InMemoryEventBus::new());

    let audio_service = Arc::new(
        RodioAudioService::new().map_err(|e| BootstrapError::AudioInit(e.to_string()))?
    );

    let audio_library_service: Arc<Mutex<dyn usecases::audio::manage_library::AudioLibraryService>> =
        Arc::new(Mutex::new(InMemoryAudioLibraryService::new()));

    let timer_service: Arc<dyn domain::timer::TimerService + Send + Sync> =
        Arc::new(TimerService::new_with_services(event_publisher.clone(), Some(app_handle.clone())));


    usecases::bootstrap(&task_repository, &config_repository, &timer_service, &event_publisher).await.map_err(|e| BootstrapError::OrchestrationError(e.to_string()))?;

    let app_started = app_lifecycle::AppStarted::new(
        1,
        "v1.0.0".to_string(),
        true,
        true,
        true,
        Some(100),
        chrono::Utc::now(),
    );
    
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
