use crate::adapters::events::mem_event_bus::EventPublisherArc;
use std::sync::Arc;
use domain::{
    AppearanceConfig, AudioConfig, Config, GeneralConfig, NotificationConfig,
    TaskId,
};
use tauri::State;
use anyhow::Context;

#[tauri::command]
pub async fn get_global_config(
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
) -> Result<Config, String> {
    config_repo.get_config().await
        .context("Failed to get global configuration")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_global_config(
    config: Config,
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
    _event_publisher: State<'_, EventPublisherArc>,
) -> Result<(), String> {
    config_repo.save_config(&config).await
        .context("Failed to save global configuration")
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_config_to_defaults(
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
    _event_publisher: State<'_, EventPublisherArc>,
) -> Result<Config, String> {
    config_repo.reset_to_defaults().await
        .context("Failed to reset configuration to defaults")
        .map_err(|e| e.to_string())
}



#[tauri::command]
pub async fn update_general_config(
    preferences: GeneralConfig,
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
) -> Result<Config, String> {
    let mut config = config_repo.get_config().await
        .context("Failed to get current configuration")
        .map_err(|e| e.to_string())?;
    config.general = preferences;

    config_repo
        .save_config(&config).await
        .context("Failed to save updated general configuration")
        .map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
pub async fn update_notification_config(
    preferences: NotificationConfig,
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
) -> Result<Config, String> {
    let mut config = config_repo.get_config().await
        .context("Failed to get current configuration")
        .map_err(|e| e.to_string())?;
    config.notification = preferences;

    config_repo
        .save_config(&config).await
        .context("Failed to save updated notification configuration")
        .map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
pub async fn update_appearance_config(
    preferences: AppearanceConfig,
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
) -> Result<Config, String> {
    let mut config = config_repo.get_config().await
        .context("Failed to get current configuration")
        .map_err(|e| e.to_string())?;
    config.appearance = preferences;

    config_repo
        .save_config(&config).await
        .context("Failed to save updated appearance configuration")
        .map_err(|e| e.to_string())?;
    Ok(config)
}

#[tauri::command]
pub async fn update_audio_config(
    _task_id: TaskId,
    _audio_config: AudioConfig,
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
) -> Result<Config, String> {
    // For now, just return the current config since we don't have task-specific audio config persistence
    config_repo.get_config().await
        .context("Failed to get current configuration")
        .map_err(|e| e.to_string())
}


#[tauri::command]
pub async fn get_effective_audio_config(
    _task_id: TaskId,
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
) -> Result<AudioConfig, String> {
    let config = config_repo.get_config().await
        .context("Failed to get effective audio configuration")
        .map_err(|e| e.to_string())?;
    Ok(config.audio)
}
