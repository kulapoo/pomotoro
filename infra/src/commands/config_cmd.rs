use crate::adapters::{
    ConfigRepository, events::mem_event_bus::EventPublisherArc,
};
use domain::{
    AppearanceConfig, AudioConfig, Config, GeneralConfig, NotificationConfig,
    TaskDefaults, TaskId,
};
use tauri::State;
use anyhow::Context;

#[tauri::command]
pub async fn get_global_config(
    config_repo: State<'_, ConfigRepository>,
) -> Result<Config, String> {
    config_repo.get_config()
        .context("Failed to get global configuration")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_global_config(
    config: Config,
    config_repo: State<'_, ConfigRepository>,
    _event_publisher: State<'_, EventPublisherArc>,
) -> Result<(), String> {
    config_repo.save_config(&config)
        .context("Failed to save global configuration")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_config_to_defaults(
    config_repo: State<'_, ConfigRepository>,
    _event_publisher: State<'_, EventPublisherArc>,
) -> Result<Config, String> {
    config_repo.reset_to_defaults()
        .context("Failed to reset configuration to defaults")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_timing_config(
    work_duration_minutes: u32,
    short_break_minutes: u32,
    long_break_minutes: u32,
    config_repo: State<'_, ConfigRepository>,
) -> Result<Config, String> {
    let mut config = config_repo.get_config()
        .context("Failed to get current configuration")
        .map_err(|e| e.to_string())?;
    config.task_defaults.work_duration =
        std::time::Duration::from_secs(work_duration_minutes as u64 * 60);
    config.task_defaults.short_break_duration =
        std::time::Duration::from_secs(short_break_minutes as u64 * 60);
    config.task_defaults.long_break_duration =
        std::time::Duration::from_secs(long_break_minutes as u64 * 60);

    config_repo
        .save_config(&config)
        .context("Failed to save updated timing configuration")
        .map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
pub async fn update_default_cycle_length(
    sessions_until_long_break: u8,
    config_repo: State<'_, ConfigRepository>,
) -> Result<Config, String> {
    let mut config = config_repo.get_config()
        .context("Failed to get current configuration")
        .map_err(|e| e.to_string())?;
    config.task_defaults.sessions_until_long_break = sessions_until_long_break;

    config_repo
        .save_config(&config)
        .with_context(|| format!("Failed to update cycle length to {} sessions", sessions_until_long_break))
        .map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
pub async fn update_general_config(
    preferences: GeneralConfig,
    config_repo: State<'_, ConfigRepository>,
) -> Result<Config, String> {
    let mut config = config_repo.get_config()
        .context("Failed to get current configuration")
        .map_err(|e| e.to_string())?;
    config.general = preferences;

    config_repo
        .save_config(&config)
        .context("Failed to save updated general configuration")
        .map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
pub async fn update_notification_config(
    preferences: NotificationConfig,
    config_repo: State<'_, ConfigRepository>,
) -> Result<Config, String> {
    let mut config = config_repo.get_config()
        .context("Failed to get current configuration")
        .map_err(|e| e.to_string())?;
    config.notification = preferences;

    config_repo
        .save_config(&config)
        .context("Failed to save updated notification configuration")
        .map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
pub async fn update_appearance_config(
    preferences: AppearanceConfig,
    config_repo: State<'_, ConfigRepository>,
) -> Result<Config, String> {
    let mut config = config_repo.get_config()
        .context("Failed to get current configuration")
        .map_err(|e| e.to_string())?;
    config.appearance = preferences;

    config_repo
        .save_config(&config)
        .context("Failed to save updated appearance configuration")
        .map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
pub async fn update_audio_config(
    _task_id: TaskId,
    _audio_config: AudioConfig,
    config_repo: State<'_, ConfigRepository>,
) -> Result<Config, String> {
    // For now, just return the current config since we don't have task-specific audio config persistence
    config_repo.get_config()
        .context("Failed to get current configuration")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_effective_task_config(
    _task_id: TaskId,
    config_repo: State<'_, ConfigRepository>,
) -> Result<TaskDefaults, String> {
    let config = config_repo.get_config()
        .context("Failed to get effective task configuration")
        .map_err(|e| e.to_string())?;
    Ok(config.task_defaults)
}

#[tauri::command]
pub async fn get_effective_audio_config(
    _task_id: TaskId,
    config_repo: State<'_, ConfigRepository>,
) -> Result<AudioConfig, String> {
    let config = config_repo.get_config()
        .context("Failed to get effective audio configuration")
        .map_err(|e| e.to_string())?;
    Ok(config.audio)
}
