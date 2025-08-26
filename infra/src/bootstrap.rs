use anyhow::{anyhow, Context, Result};
use domain::EventPublisher;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;
use usecases::{get_config, timer::switch_timer_task};

use crate::adapters::{
    InMemoryConfigRepository, InMemoryEventBus,
    events::{
        EventSubscriber, app_lifecycle, mem_event_bus::EventPublisherArc,
    },
    task::{InMemoryTaskRepository, register_task_handlers},
    timer::event_handlers::register_timer_handlers,
};
use tauri::AppHandle;

use crate::adapters::{
    InMemoryTimerService, RodioAudioService, audio::InMemoryAudioLibraryService,
};
use domain::timer::TimerService;

pub struct AppRegistry {
    pub task_repository: Arc<dyn domain::TaskRepository + Send + Sync>,
    pub config_repository: Arc<dyn domain::ConfigRepository + Send + Sync>,
    pub event_publisher: EventPublisherArc,
    pub timer_service: Arc<dyn TimerService + Send + Sync>,
    pub audio_service: Arc<RodioAudioService>,
    #[allow(dead_code)]
    pub audio_library_service:
        Arc<Mutex<dyn usecases::audio::manage_library::AudioLibraryService>>,
}

pub fn register_handlers(
    event_bus: Arc<dyn EventSubscriber + Send + Sync>,
    app_handle: AppHandle,
) -> Result<()> {
    register_timer_handlers(event_bus.clone(), app_handle.clone())
        .context("Failed to register timer event handlers")?;
    register_task_handlers(event_bus, app_handle)
        .context("Failed to register task event handlers")?;
    Ok(())
}

pub async fn bootstrap(app_handle: AppHandle) -> Result<AppRegistry> {
    let config_repository: Arc<dyn domain::ConfigRepository + Send + Sync> =
        Arc::new(InMemoryConfigRepository::default());

    let config = get_config(&config_repository)
        .await
        .context("Failed to initialize configuration")?;

    let task_defaults = config.task_defaults;
    let task_repository: Arc<dyn domain::TaskRepository + Send + Sync> =
        Arc::new(InMemoryTaskRepository::with_default_task(&task_defaults));

    let default_task = task_repository
        .get_default_task()
        .await
        .context("Failed to get default task")?
        .ok_or(anyhow!("No default task found"))?;

    info!("Bootstraping Pomotoro...");

    info!(
        "Default task defaults: {:?}",
        default_task.description
    );

    let event_bus = Arc::new(InMemoryEventBus::new());
    let event_publisher: Arc<dyn EventPublisher + Send + Sync + 'static> =
        event_bus.clone();

    register_handlers(event_bus.clone(), app_handle.clone())?;

    let audio_service = Arc::new(
        RodioAudioService::new()
            .context("Failed to initialize audio service")?,
    );

    let audio_library_service: Arc<
        Mutex<dyn usecases::audio::manage_library::AudioLibraryService>,
    > = Arc::new(Mutex::new(InMemoryAudioLibraryService::new()));

    let timer_service: Arc<dyn TimerService + Send + Sync> =
        Arc::new(InMemoryTimerService::new_with_services(
            event_publisher.clone(),
            Some(app_handle.clone()),
        ));

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
        audio_library_service,
    };

    Ok(ctx)
}
