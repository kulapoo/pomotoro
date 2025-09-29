use anyhow::{Context, Result};
use domain::EventPublisher;
use std::sync::Arc;
use tracing::info;

use crate::adapters::{
    InMemoryEventBus, RodioAudioService, SqliteConfigRepository,
    SqliteTaskRepository, SqliteTimerRepository, TimerTickService,
    audio::{AudioServiceWrapper, register_audio_event_handlers},
    config::register_config_handlers,
    establish_connection,
    events::{
        EventSubscriber,
        app_emitter::{Emitter, TauriAppHandleEmitter},
        app_started_handler::AppStartedHandler,
        mem_event_bus::EventPublisherArc,
    },
    notifications::register_notification_handlers,
    run_migrations,
    task::{DefaultCyclingService, register_task_handlers},
    timer::event_handlers::register_timer_handlers,
};
use domain::TimerRepository;
use tauri::AppHandle;

pub struct AppRegistry {
    pub task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
    pub task_cycling_service: Arc<dyn domain::TaskCyclerService + Send + Sync>,
    pub config_repository: Arc<dyn domain::ConfigRepository + Send + Sync>,
    pub event_publisher: EventPublisherArc,
    pub timer_tick_service: Arc<TimerTickService>,
    pub timer_repository: Arc<dyn TimerRepository + Send + Sync>,
    pub audio_service: Arc<AudioServiceWrapper>,
}

pub async fn register_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    app_handle: AppHandle,
    config_repository: Arc<dyn domain::ConfigRepository + Send + Sync>,
    task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
    timer_tick_service: Arc<TimerTickService>,
    audio_service: Arc<AudioServiceWrapper>,
) -> Result<()> {
    // Create the emitter that will be shared by all event handlers
    let emitter: Arc<dyn Emitter> =
        Arc::new(TauriAppHandleEmitter::new(app_handle.clone()));

    event_bus.subscribe(Box::new(AppStartedHandler::new(emitter.clone())))?;

    register_timer_handlers(
        event_bus.clone(),
        emitter.clone(),
        timer_tick_service.clone(),
        task_repository.clone(),
        config_repository.clone(),
    )
    .context("Failed to register timer event handlers")?;
    register_task_handlers(event_bus.clone(), emitter.clone())
        .context("Failed to register task event handlers")?;
    register_config_handlers(event_bus.clone(), emitter.clone())
        .context("Failed to register config event handlers")?;
    register_notification_handlers(
        event_bus.clone(),
        app_handle,
        config_repository.clone(),
        task_repository,
    )
    .await
    .inspect_err(|e| {
        eprintln!("Error in register_notification_handlers: {:?}", e)
    })
    .context("Failed to register notification event handlers")?;
    register_audio_event_handlers(event_bus, audio_service, config_repository)
        .context("Failed to register audio event handlers")?;

    Ok(())
}

pub async fn bootstrap(app_handle: AppHandle) -> Result<AppRegistry> {
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

    info!("Bootstraping Pomotoro...");

    let event_bus = Arc::new(InMemoryEventBus::new());
    let event_publisher: Arc<dyn EventPublisher + Send + Sync + 'static> =
        event_bus.clone();

    let audio_service = Arc::new(AudioServiceWrapper::new(Box::new(
        RodioAudioService::new()
            .context("Failed to initialize audio service")?,
    )));

    let task_cycling_service: Arc<dyn domain::TaskCyclerService + Send + Sync> =
        Arc::new(DefaultCyclingService::new(task_repository.clone()));

    // Create timer repository
    let timer_repository: Arc<dyn TimerRepository + Send + Sync> =
        Arc::new(SqliteTimerRepository::new(db_pool.clone()));

    let timer_tick_service = Arc::new(TimerTickService::new(
        event_publisher.clone(),
        timer_repository.clone(),
        task_repository.clone(),
        config_repository.clone(),
    ));

    // Register all event handlers
    register_handlers(
        event_bus.clone(),
        app_handle.clone(),
        config_repository.clone(),
        task_repository.clone(),
        timer_tick_service.clone(),
        audio_service.clone(),
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
        eprintln!("Bootstrap error chain:");
        eprintln!("  Root cause: {:?}", e);
        eprintln!("  Display: {}", e);
        anyhow::anyhow!("Failed to bootstrap application: {}", e)
    })?;

    let ctx = AppRegistry {
        task_repository,
        task_cycling_service,
        config_repository,
        event_publisher,
        timer_tick_service,
        timer_repository,
        audio_service,
    };

    info!("Bootstrap complete");

    Ok(ctx)
}
