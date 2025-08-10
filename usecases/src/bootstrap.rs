use std::sync::Arc;

use domain::{events::{self}, timer::TimerService, ConfigRepository, Error, EventPublisher, Result, TaskRepository};

use crate::{config, timer::{start_timer_session, StartTimerSessionCmd}};

pub async fn bootstrap(
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
    timer_service: &Arc<dyn TimerService + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<()> {
    let _config = config::get_config(config_repo).await?;

    let task = task_repo.get_default_task().await?.ok_or(Error::DefaultTaskNotFound)?;

    let start_session_cmd = StartTimerSessionCmd {
        task_id: Some(task.id.to_string()),
    };

    start_timer_session(timer_service, task_repo, event_publisher, start_session_cmd).await?;


    let app_started = events::app::AppStarted {
        app_version: "v1.0.0".to_string(),
        default_task_created: true,
        config_loaded: true,
        occurred_at: chrono::Utc::now(),
        startup_duration_ms: Some(1000),
        timer_auto_started: true,
        version: 1,
    };

    event_publisher.publish(Box::new(app_started));

    Ok(())
}