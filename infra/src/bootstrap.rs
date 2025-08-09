use std::sync::Arc;
use thiserror::Error;

use domain::{InMemoryTaskRepository, WorkSessionCompleted};
use tauri::AppHandle;

use crate::adapters::{
    create_event_publisher_with_bus, ConfigRepository, DomainEventBus, EventPublisherArc,
    FileConfigRepo, RodioAudioService, TaskRepositoryArc, TimerService,
};

pub struct AppRegistry {
    pub task_repository: TaskRepositoryArc,
    pub config_repository: ConfigRepository,

    pub event_bus: Arc<DomainEventBus>,
    pub event_publisher: EventPublisherArc,

    pub timer_service: TimerService,
    pub audio_service: RodioAudioService,
}

fn init_events(app_handle: AppHandle, event_bus: Arc<DomainEventBus>) -> Result<(), BootstrapError> {
    tokio::spawn(async move {
        
    });

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
}

impl From<BootstrapError> for String {
    fn from(err: BootstrapError) -> Self {
        err.to_string()
    }
}

pub fn bootstrap(app_handle: AppHandle) -> Result<AppRegistry, BootstrapError> {
    let task_repository: TaskRepositoryArc = Arc::new(InMemoryTaskRepository::with_default_task());
    let config_repository: ConfigRepository = Arc::new(
        FileConfigRepo::new(&app_handle).map_err(|e| BootstrapError::ConfigInit(e.to_string()))?,
    );

    let (event_publisher, event_bus): (EventPublisherArc, Arc<DomainEventBus>) =
        create_event_publisher_with_bus(app_handle.clone());

    let audio_service =
        RodioAudioService::new().map_err(|e| BootstrapError::AudioInit(e.to_string()))?;

    let timer_service =
        TimerService::new_with_services(event_publisher.clone(), Some(app_handle.clone()));

    let ctx = AppRegistry {
        task_repository,
        config_repository,
        event_bus,
        event_publisher,
        timer_service,
        audio_service,
    };

    Ok(ctx)
}
