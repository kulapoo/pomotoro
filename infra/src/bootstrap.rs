use anyhow::{anyhow, Context, Result};
use domain::EventPublisher;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use usecases::{timer::switch_timer_task};

use crate::adapters::{
    FileConfigRepository, InMemoryEventBus,
    FileTaskRepository, FileStorageService, StorageConfig, FileTimerService,
    RodioAudioService, audio::{AudioServiceWrapper, InMemoryAudioLibraryService, register_audio_event_handlers},
    events::{
        EventSubscriber, app_lifecycle, mem_event_bus::EventPublisherArc,
    },
    task::{register_task_handlers, StandardTaskCyclerService},
    timer::event_handlers::register_timer_handlers,
    notifications::register_notification_handlers,
};
use tauri::AppHandle;
use domain::timer::TimerService;

pub struct AppRegistry {
    pub task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
    pub config_repository: Arc<dyn domain::ConfigRepository + Send + Sync>,
    pub event_publisher: EventPublisherArc,
    pub timer_service: Arc<dyn TimerService + Send + Sync>,
    pub audio_service: Arc<AudioServiceWrapper>,
    pub task_cycling_service: Arc<dyn domain::TaskCyclerService + Send + Sync>,
    #[allow(dead_code)]
    pub audio_library_service:
        Arc<Mutex<dyn usecases::audio::manage_library::AudioLibraryService>>,
}

pub fn register_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    app_handle: AppHandle,
    config_repository: Arc<dyn domain::ConfigRepository + Send + Sync>,
    task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
    audio_service: Arc<AudioServiceWrapper>,
) -> Result<()> {
    register_timer_handlers(event_bus.clone(), app_handle.clone())
        .context("Failed to register timer event handlers")?;
    register_task_handlers(event_bus.clone(), app_handle.clone())
        .context("Failed to register task event handlers")?;
    register_notification_handlers(
        event_bus.clone(),
        app_handle,
        config_repository.clone(),
        task_repository,
    ).context("Failed to register notification event handlers")?;
    register_audio_event_handlers(
        event_bus,
        audio_service,
        config_repository,
    ).context("Failed to register audio event handlers")?;
    Ok(())
}

pub async fn bootstrap(app_handle: AppHandle) -> Result<AppRegistry> {
    let storage_config = StorageConfig::default();
    let storage_service = FileStorageService::new(storage_config)
        .context("Failed to initialize storage service")?;

    let storage_path = storage_service.get_storage_path().await;

    let config_file = storage_path.join("Config.json");
    let config_repository: Arc<dyn domain::ConfigRepository + Send + Sync> =
        Arc::new(FileConfigRepository::new(config_file));

    let tasks_file = storage_path.join("tasks.json");

    let task_repository: Arc<dyn domain::TaskRepository + Send + Sync> =
        Arc::new(FileTaskRepository::new(tasks_file));

    let default_task = if let Some(task) = task_repository.get_default_task().await? {
        task
    } else {
        let task = domain::Task::new("Default Task".to_string(), 4)
            .map_err(|e| anyhow!("Failed to create default task: {}", e))?
            .with_default(true);
        task_repository.create(task.clone()).await
            .context("Failed to save default task")?;
        task
    };

    info!("Bootstraping Pomotoro...");

    info!(
        "Default task: {}",
        default_task.name
    );

    let event_bus = Arc::new(InMemoryEventBus::new());
    let event_publisher: Arc<dyn EventPublisher + Send + Sync + 'static> =
        event_bus.clone();

    let audio_service = Arc::new(AudioServiceWrapper::new(
        Box::new(
            RodioAudioService::new()
                .context("Failed to initialize audio service")?,
        )
    ));

    let audio_library_service: Arc<
        Mutex<dyn usecases::audio::manage_library::AudioLibraryService>,
    > = Arc::new(Mutex::new(InMemoryAudioLibraryService::new()));

    // Register all event handlers
    register_handlers(
        event_bus.clone(),
        app_handle.clone(),
        config_repository.clone(),
        task_repository.clone(),
        audio_service.clone(),
    )?;

    let task_cycling_service: Arc<dyn domain::TaskCyclerService + Send + Sync> =
        Arc::new(StandardTaskCyclerService::new(
            task_repository.clone(),
            domain::TaskCyclingStrategy::RoundRobin,
        ));

    let timer_service: Arc<dyn TimerService + Send + Sync> =
        Arc::new(FileTimerService::new(
            event_publisher.clone(),
            Some(storage_path.clone()),
            config_repository.clone(),
        ));

    timer_service.load_state().await
        .context("Failed to load timer state")?;

    switch_timer_task(
        &timer_service,
        &task_repository,
        &event_publisher,
        switch_timer_task::SwitchTimerTaskCmd {
            task_id: default_task.id.to_string()
        },
    ).await?;

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
        task_cycling_service,
        audio_library_service,
    };

    Ok(ctx)
}
