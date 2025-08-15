use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::adapters::{task::InMemoryTaskRepository, InMemoryConfigRepository};
use tauri::AppHandle;
use usecases::HandlerRegistry;

use crate::adapters::{
    create_event_publisher_with_bus, DomainEventBus, EventPublisherArc,
    RodioAudioService, TimerService,
    timer::handlers::{
        PhaseCompletionHandler, ConcreteAudioNotificationPlayer, TauriNotificationService,
    },
    audio::InMemoryAudioLibraryService,
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
    #[allow(dead_code)]
    pub audio_library_service: Arc<Mutex<dyn usecases::audio::manage_library::AudioLibraryService>>,
}

fn init_events(
    event_handlers: &mut HandlerRegistry,
    app_handle: &AppHandle,
    audio_service: Arc<Mutex<dyn domain::AudioService>>,
    audio_library_service: Arc<Mutex<dyn usecases::audio::manage_library::AudioLibraryService>>,
) -> Result<(), BootstrapError> {
    // Create the audio notification player
    let audio_player = Arc::new(RwLock::new(
        ConcreteAudioNotificationPlayer::new(audio_service, audio_library_service)
    ));
    
    // Create the notification service
    let notification_service = Arc::new(TauriNotificationService::new(app_handle.clone()));
    
    // Create and register the PhaseCompletionHandler
    let phase_completion_handler = Box::new(
        PhaseCompletionHandler::new(audio_player, notification_service)
    );
    
    event_handlers.register(phase_completion_handler);

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
    
    let audio_library_service: Arc<Mutex<dyn usecases::audio::manage_library::AudioLibraryService>> = 
        Arc::new(Mutex::new(InMemoryAudioLibraryService::new()));

    let timer_service: Arc<dyn domain::timer::TimerService + Send + Sync> =
        Arc::new(TimerService::new_with_services(event_publisher.clone(), Some(app_handle.clone())));

    let mut event_handlers = HandlerRegistry::new();
    
    // Initialize event handlers with the PhaseCompletionHandler
    let audio_service_for_handler: Arc<Mutex<dyn domain::AudioService>> = 
        Arc::new(Mutex::new(RodioAudioService::new().map_err(|e| BootstrapError::AudioInit(e.to_string()))?));
        
    init_events(
        &mut event_handlers, 
        &app_handle,
        audio_service_for_handler,
        audio_library_service.clone()
    ).map_err(|e| BootstrapError::EventSystem(e.to_string()))?;

    usecases::bootstrap(&task_repository, &config_repository, &timer_service, &event_publisher).await.map_err(|e| BootstrapError::OrchestrationError(e.to_string()))?;

    let ctx = AppRegistry {
        event_handlers: Arc::new(event_handlers),
        task_repository,
        config_repository,
        event_bus,
        event_publisher,
        timer_service,
        audio_service,
        audio_library_service,
    };

    Ok(ctx)
}
