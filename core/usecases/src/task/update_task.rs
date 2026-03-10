use domain::{
    AudioConfig, Error, EventPublisher, Result, Task, TaskId, TaskRepository,
    TaskUpdated,
};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Default, Clone)]
pub struct UpdateTaskCmd {
    pub id: TaskId,
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_sessions: Option<u8>,
    pub tags: Option<Vec<String>>,
    pub work_duration: Option<Duration>,
    pub short_break_duration: Option<Duration>,
    pub long_break_duration: Option<Duration>,
    pub sessions_until_long_break: Option<u8>,
    pub enable_screen_blocking: Option<bool>,
    pub audio_config: Option<AudioConfig>,
}

pub async fn update_task(
    task_repo: Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: UpdateTaskCmd,
) -> Result<Task> {
    let mut task = task_repo.get_by_id(cmd.id).await?.ok_or_else(|| {
        Error::TaskNotFound {
            id: cmd.id.to_string(),
        }
    })?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    let updated_name = cmd.name.clone();
    let updated_description = cmd.description.clone();
    let updated_max_sessions = cmd.max_sessions;
    let updated_tags = cmd.tags.clone();

    if let Some(name) = cmd.name {
        task.set_name(name)?;
    }

    if let Some(description) = cmd.description {
        task.set_description(description);
    }

    if let Some(max_sessions) = cmd.max_sessions {
        task.set_max_sessions(max_sessions)?;
    }

    if let Some(tags) = cmd.tags {
        task.set_tags(tags);
    }

    // Update timer settings if any provided (using builder methods for validation)
    if let Some(work_duration) = cmd.work_duration {
        let new_timer =
            task.config().timer.with_work_duration(work_duration)?;
        task.config_mut().timer = new_timer;
    }
    if let Some(short_break_duration) = cmd.short_break_duration {
        let new_timer = task
            .config()
            .timer
            .with_short_break_duration(short_break_duration)?;
        task.config_mut().timer = new_timer;
    }
    if let Some(long_break_duration) = cmd.long_break_duration {
        let new_timer = task
            .config()
            .timer
            .with_long_break_duration(long_break_duration)?;
        task.config_mut().timer = new_timer;
    }
    if let Some(sessions_until_long_break) = cmd.sessions_until_long_break {
        let new_timer = task
            .config()
            .timer
            .with_sessions_until_long_break(sessions_until_long_break)?;
        task.config_mut().timer = new_timer;
    }
    if let Some(enable_screen_blocking) = cmd.enable_screen_blocking {
        task.config_mut().general.enable_screen_blocking =
            enable_screen_blocking;
    }

    if let Some(audio_config) = cmd.audio_config {
        audio_config.validate()?;
        task.config_mut().audio = audio_config;
    }

    task_repo.update(task.clone()).await?;

    let updated_event = TaskUpdated::new(
        task.id(),
        updated_name,
        updated_description,
        updated_max_sessions,
        updated_tags,
        1,
    );
    event_publisher.publish(Box::new(updated_event));

    Ok(task)
}
