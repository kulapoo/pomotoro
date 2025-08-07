use std::sync::Arc;

use domain::InMemoryTaskRepository;
use tauri::AppHandle;

use crate::adapters::{create_event_publisher_with_bus, ConfigRepository, DomainEventBus, EventPublisherArc, FileConfigRepo, RodioAudioService, TaskRepositoryArc, TimerService};

struct AppRegistry {
    pub task_repository: TaskRepositoryArc,
    pub config_repository: ConfigRepository,

    pub event_bus: Arc<DomainEventBus>,
    pub event_publisher: EventPublisherArc,

    pub timer_service: TimerService,
    pub audio_service:  RodioAudioService
}

pub fn boostrap(app_handle: AppHandle) -> AppRegistry {
    // repositories
    let task_repository: TaskRepositoryArc = Arc::new(InMemoryTaskRepository::with_default_task());
    let config_repository: ConfigRepository = Arc::new(
        FileConfigRepo::new(&app_handle).expect("Failed to initialize config repository"),
    );


    // events
    let (event_publisher, event_bus): (EventPublisherArc, Arc<DomainEventBus>) =
        create_event_publisher_with_bus(app_handle.clone());


    // services
    let audio_service =
        RodioAudioService::new().expect("Failed to initialize audio service");

    let timer_service = TimerService::new_with_services(
        event_publisher.clone(),
        Some(app_handle.clone()),
    );

    let ctx = AppRegistry {
        task_repository,
        config_repository,
        event_bus,
        event_publisher,
        timer_service,
        audio_service
    };

    ctx
}