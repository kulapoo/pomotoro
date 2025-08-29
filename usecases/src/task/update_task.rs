use domain::{
    AudioConfig, Error, EventPublisher, Result, Task, TaskId,
    TaskRepository, TaskUpdated,
};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct UpdateTaskCmd {
    pub id: String,
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
    task_repo: &Arc<dyn TaskRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    cmd: UpdateTaskCmd,
) -> Result<Task> {
    let task_id = TaskId::from_string(&cmd.id)
        .map_err(|_| Error::TaskNotFound { id: cmd.id.clone() })?;

    let mut task = task_repo
        .get_by_id(task_id)
        .await?
        .ok_or_else(|| Error::TaskNotFound { id: cmd.id.clone() })?;

    if task.is_completed() {
        return Err(Error::TaskAlreadyCompleted);
    }

    let updated_name = cmd.name.clone();
    let updated_description = cmd.description.clone();
    let updated_max_sessions = cmd.max_sessions;
    let updated_tags = cmd.tags.clone();

    if let Some(name) = cmd.name {
        if name.trim().is_empty() {
            return Err(Error::InvalidSessionCount { count: 0 });
        }
        task.name = name.trim().to_string();
    }

    if let Some(description) = cmd.description {
        task.description = Some(description);
    }

    if let Some(max_sessions) = cmd.max_sessions {
        if max_sessions == 0 {
            return Err(Error::InvalidSessionCount {
                count: max_sessions,
            });
        }
        task.max_sessions = max_sessions;

        if task.current_sessions >= max_sessions {
            task.status = domain::TaskStatus::Completed;
            task.completed_at = Some(chrono::Utc::now());
        }
    }

    if let Some(tags) = cmd.tags {
        task.tags = tags;
    }

    // Update timer settings if any provided
    if cmd.work_duration.is_some() || 
       cmd.short_break_duration.is_some() || 
       cmd.long_break_duration.is_some() ||
       cmd.sessions_until_long_break.is_some() ||
       cmd.enable_screen_blocking.is_some() {
        task.settings.update_timer_settings(
            cmd.work_duration,
            cmd.short_break_duration,
            cmd.long_break_duration,
            cmd.sessions_until_long_break,
            cmd.enable_screen_blocking,
        )?;
    }

    if let Some(audio_config) = cmd.audio_config {
        audio_config.validate()?;
        task.audio_config = audio_config;
    }

    task_repo.update(task.clone()).await?;

    let updated_event = TaskUpdated::new(
        task.id,
        updated_name,
        updated_description,
        updated_max_sessions,
        updated_tags,
        1,
    );
    event_publisher.publish(Box::new(updated_event));

    Ok(task)
}

