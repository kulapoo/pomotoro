use std::sync::Arc;

use domain::{ConfigRepository, Error, EventPublisher, Result, TaskRepository};
use domain::timer::TimerService;

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

    Ok(())
}