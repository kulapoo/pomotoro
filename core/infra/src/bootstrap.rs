use anyhow::{Context, Result};
use domain::EventPublisher;
use log::info;
use std::sync::Arc;

use crate::adapters::{
    InMemoryEventBus, RodioAudioService, SqliteConfigRepository,
    SqliteTaskRepository, SqliteTimerRepository, TimerTickService,
    audio::{AudioServiceWrapper, register_audio_event_handlers},
    config::register_config_handlers,
    establish_connection,
    events::{
        EventSubscriber, app_emitter::Emitter,
        app_started_handler::AppStartedHandler,
        mem_event_bus::EventPublisherArc,
    },
    notifications::{NotificationServiceTrait, register_notification_handlers},
    run_migrations,
    task::register_task_handlers,
    timer::event_handlers::register_timer_handlers,
};
use domain::TimerRepository;

pub struct AppRegistry {
    pub task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
    pub config_repository: Arc<dyn domain::ConfigRepository + Send + Sync>,
    pub event_publisher: EventPublisherArc,
    pub timer_tick_service: Arc<TimerTickService>,
    pub timer_repository: Arc<dyn TimerRepository + Send + Sync>,
    pub audio_service: Arc<AudioServiceWrapper>,
}

#[allow(clippy::too_many_arguments)]
pub async fn register_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    emitter: Arc<dyn Emitter>,
    notification_service: Arc<dyn NotificationServiceTrait>,
    config_repository: Arc<dyn domain::ConfigRepository + Send + Sync>,
    task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
    timer_repository: Arc<dyn domain::TimerRepository + Send + Sync>,
    timer_tick_service: Arc<TimerTickService>,
    audio_service: Arc<AudioServiceWrapper>,
    event_publisher: Arc<dyn domain::EventPublisher + Send + Sync>,
) -> Result<()> {
    event_bus.subscribe(Box::new(AppStartedHandler::new(emitter.clone())))?;

    register_timer_handlers(
        event_bus.clone(),
        emitter.clone(),
        timer_tick_service.clone(),
        task_repository.clone(),
        timer_repository.clone(),
        config_repository.clone(),
        event_publisher.clone(),
    )
    .context("Failed to register timer event handlers")?;

    register_task_handlers(
        event_bus.clone(),
        emitter.clone(),
        task_repository.clone(),
        timer_tick_service.clone(),
    )
    .context("Failed to register task event handlers")?;

    register_config_handlers(event_bus.clone(), emitter.clone())
        .context("Failed to register config event handlers")?;

    register_notification_handlers(
        event_bus.clone(),
        notification_service,
        task_repository,
    )
    .await
    .inspect_err(|e| {
        log::error!("Error in register_notification_handlers: {:?}", e)
    })
    .context("Failed to register notification event handlers")?;

    register_audio_event_handlers(event_bus, audio_service, config_repository)
        .context("Failed to register audio event handlers")?;

    Ok(())
}

pub async fn bootstrap(
    emitter: Arc<dyn Emitter>,
    notification_service: Arc<dyn NotificationServiceTrait>,
) -> Result<AppRegistry> {
    // Get default storage path for database
    let storage_path = dirs::data_dir()
        .context("Failed to get user data directory")?
        .join("pomotoro");

    std::fs::create_dir_all(&storage_path)
        .context("Failed to create storage directory")?;

    // Set up SQLite database
    let db_path = storage_path.join("pomotoro.db");
    let db_pool = Arc::new(
        establish_connection(&db_path)
            .context("Failed to establish database connection")?,
    );

    // Run migrations
    run_migrations(&db_pool).context("Failed to run database migrations")?;

    let config_repository: Arc<dyn domain::ConfigRepository + Send + Sync> =
        Arc::new(SqliteConfigRepository::new(db_pool.clone()));

    let task_repository: Arc<dyn domain::TaskRepository + Send + Sync> =
        Arc::new(SqliteTaskRepository::new(db_pool.clone()));

    info!("Bootstrapping Pomotoro...");

    let event_bus = Arc::new(InMemoryEventBus::new());
    let event_publisher: Arc<dyn EventPublisher + Send + Sync + 'static> =
        event_bus.clone();

    let audio_service = Arc::new(AudioServiceWrapper::new(Box::new(
        RodioAudioService::new()
            .context("Failed to initialize audio service")?,
    )));

    // Create timer repository
    let timer_repository: Arc<dyn TimerRepository + Send + Sync> =
        Arc::new(SqliteTimerRepository::new(db_pool.clone()));

    let timer_tick_service = Arc::new(TimerTickService::new(
        event_publisher.clone(),
        timer_repository.clone(),
        config_repository.clone(),
    ));

    // Register all event handlers
    register_handlers(
        event_bus.clone(),
        emitter,
        notification_service,
        config_repository.clone(),
        task_repository.clone(),
        timer_repository.clone(),
        timer_tick_service.clone(),
        audio_service.clone(),
        event_publisher.clone(),
    )
    .await?;

    usecases::bootstrap(
        timer_repository.clone(),
        task_repository.clone(),
        config_repository.clone(),
        event_publisher.clone(),
    )
    .await
    .map_err(|e| {
        log::error!("Bootstrap error chain:");
        log::error!("  Root cause: {:?}", e);
        log::error!("  Display: {}", e);
        anyhow::anyhow!("Failed to bootstrap application: {}", e)
    })?;

    let ctx = AppRegistry {
        task_repository,
        config_repository,
        event_publisher,
        timer_tick_service,
        timer_repository,
        audio_service,
    };

    info!("Bootstrap complete");

    Ok(ctx)
}
