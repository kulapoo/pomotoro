use domain::{
    ConfigRepository, Result, TaskId, TaskRepository,
};
use std::sync::Arc;

/// Get task configuration with resolved settings.
/// Returns a serializable struct containing all resolved values.
pub async fn get_effective_task_settings(
    task_repository: &Arc<dyn TaskRepository + Send + Sync>,
    _config_repository: &Arc<dyn ConfigRepository + Send + Sync>,
    task_id: TaskId,
) -> Result<ResolvedTaskSettings> {
    let task = task_repository.get_by_id(task_id).await?
        .ok_or(domain::Error::TaskNotFound { id: task_id.to_string() })?;
    
    // Create a serializable struct with all resolved values from task config
    Ok(ResolvedTaskSettings {
        max_sessions: task.max_sessions,
        work_duration: task.config.timer.work_duration,
        short_break_duration: task.config.timer.short_break_duration,
        long_break_duration: task.config.timer.long_break_duration,
        sessions_until_long_break: task.config.timer.sessions_until_long_break,
        enable_screen_blocking: task.config.general.enable_screen_blocking,
        audio_config: task.config.audio.clone(),
        notification_config: task.config.notification.clone(),
    })
}

/// Serializable struct containing resolved task settings
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResolvedTaskSettings {
    pub max_sessions: u8,
    #[serde(with = "domain::duration_serde")]
    pub work_duration: std::time::Duration,
    #[serde(with = "domain::duration_serde")]
    pub short_break_duration: std::time::Duration,
    #[serde(with = "domain::duration_serde")]
    pub long_break_duration: std::time::Duration,
    pub sessions_until_long_break: u8,
    pub enable_screen_blocking: bool,
    pub audio_config: domain::AudioConfig,
    pub notification_config: domain::NotificationConfig,
}

