use crate::adapters::events::mem_event_bus::EventPublisherArc;
use anyhow::Context;
use domain::{
    AppearanceConfig, AudioConfig, Config, GeneralConfig, NotificationConfig,
    TaskId,
};
use std::sync::Arc;
use tauri::State;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_global_config(
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
) -> Result<Config, String> {
    log::info!("Getting global configuration");

    config_repo
        .get_config()
        .await
        .context("Failed to get global configuration")
        .map_err(|e| {
            log::error!("Failed to get global config: {}", e);
            e.to_string()
        })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn save_global_config(
    config: Config,
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<(), String> {
    log::info!("Saving global configuration");

    // Validate before saving
    config.validate().map_err(|e| {
        log::error!("Config validation failed: {}", e);
        e.to_string()
    })?;

    config_repo
        .save_config(&config)
        .await
        .context("Failed to save global configuration")
        .map_err(|e| {
            log::error!("Failed to save global config: {}", e);
            e.to_string()
        })?;

    log::info!("Global configuration saved successfully");

    // Publish ConfigUpdated event
    let config_updated = domain::ConfigUpdated::new(config);
    event_publisher.publish(Box::new(config_updated));

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_config_to_defaults(
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Config, String> {
    log::info!("Resetting configuration to defaults");

    let config = config_repo
        .reset_to_defaults()
        .await
        .context("Failed to reset configuration to defaults")
        .map_err(|e| {
            log::error!("Failed to reset config: {}", e);
            e.to_string()
        })?;

    log::info!("Configuration reset to defaults successfully");

    // Publish ConfigReset event
    let config_reset = domain::ConfigReset::new(config.clone());
    event_publisher.publish(Box::new(config_reset));

    Ok(config)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_general_config(
    preferences: GeneralConfig,
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Config, String> {
    log::info!("Updating general configuration: {:?}", preferences);

    let mut config = config_repo
        .get_config()
        .await
        .context("Failed to get current configuration")
        .map_err(|e| {
            log::error!("Failed to get current config: {}", e);
            e.to_string()
        })?;

    config.general = preferences;

    // Validate before saving
    config.validate().map_err(|e| {
        log::error!("Config validation failed: {}", e);
        e.to_string()
    })?;

    config_repo
        .save_config(&config)
        .await
        .context("Failed to save updated general configuration")
        .map_err(|e| {
            log::error!("Failed to save config: {}", e);
            e.to_string()
        })?;

    log::info!("General configuration updated successfully");

    // Publish ConfigUpdated event
    let config_updated = domain::ConfigUpdated::new(config.clone());
    event_publisher.publish(Box::new(config_updated));

    Ok(config)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_notification_config(
    preferences: NotificationConfig,
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Config, String> {
    log::info!("Updating notification configuration: {:?}", preferences);

    let mut config = config_repo
        .get_config()
        .await
        .context("Failed to get current configuration")
        .map_err(|e| {
            log::error!("Failed to get current config: {}", e);
            e.to_string()
        })?;

    config.notification = preferences;

    // Validate before saving
    config.validate().map_err(|e| {
        log::error!("Config validation failed: {}", e);
        e.to_string()
    })?;

    config_repo
        .save_config(&config)
        .await
        .context("Failed to save updated notification configuration")
        .map_err(|e| {
            log::error!("Failed to save config: {}", e);
            e.to_string()
        })?;

    log::info!("Notification configuration updated successfully");

    // Publish ConfigUpdated event
    let config_updated = domain::ConfigUpdated::new(config.clone());
    event_publisher.publish(Box::new(config_updated));

    Ok(config)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_appearance_config(
    preferences: AppearanceConfig,
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Config, String> {
    log::info!("Updating appearance configuration: {:?}", preferences);

    let mut config = config_repo
        .get_config()
        .await
        .context("Failed to get current configuration")
        .map_err(|e| {
            log::error!("Failed to get current config: {}", e);
            e.to_string()
        })?;

    config.appearance = preferences;

    // Validate before saving
    config.validate().map_err(|e| {
        log::error!("Config validation failed: {}", e);
        e.to_string()
    })?;

    config_repo
        .save_config(&config)
        .await
        .context("Failed to save updated appearance configuration")
        .map_err(|e| {
            log::error!("Failed to save config: {}", e);
            e.to_string()
        })?;

    log::info!("Appearance configuration updated successfully");

    // Publish ConfigUpdated event
    let config_updated = domain::ConfigUpdated::new(config.clone());
    event_publisher.publish(Box::new(config_updated));

    Ok(config)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_audio_config(
    _task_id: TaskId,
    audio_config: AudioConfig,
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Config, String> {
    log::info!("Updating audio configuration: {:?}", audio_config);

    let mut config = config_repo
        .get_config()
        .await
        .context("Failed to get current configuration")
        .map_err(|e| {
            log::error!("Failed to get current config: {}", e);
            e.to_string()
        })?;

    config.audio = audio_config;

    // Validate before saving
    config.validate().map_err(|e| {
        log::error!("Config validation failed: {}", e);
        e.to_string()
    })?;

    config_repo
        .save_config(&config)
        .await
        .context("Failed to save updated audio configuration")
        .map_err(|e| {
            log::error!("Failed to save config: {}", e);
            e.to_string()
        })?;

    log::info!("Audio configuration updated successfully");

    // Publish ConfigUpdated event
    let config_updated = domain::ConfigUpdated::new(config.clone());
    event_publisher.publish(Box::new(config_updated));

    Ok(config)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_effective_audio_config(
    _task_id: TaskId,
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
) -> Result<AudioConfig, String> {
    log::info!("Getting effective audio configuration");

    let config = config_repo
        .get_config()
        .await
        .context("Failed to get effective audio configuration")
        .map_err(|e| {
            log::error!("Failed to get effective audio config: {}", e);
            e.to_string()
        })?;

    Ok(config.audio)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn update_timing_config(
    timer: domain::TimerConfiguration,
    config_repo: State<'_, Arc<dyn domain::ConfigRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Config, String> {
    log::info!("Updating timer configuration: {:?}", timer);

    let mut config = config_repo
        .get_config()
        .await
        .context("Failed to get current configuration")
        .map_err(|e| {
            log::error!("Failed to get current config: {}", e);
            e.to_string()
        })?;

    config.timer = timer;

    // Validate the config before saving
    config.validate().map_err(|e| {
        log::error!("Config validation failed: {}", e);
        e.to_string()
    })?;

    config_repo
        .save_config(&config)
        .await
        .context("Failed to save updated timer configuration")
        .map_err(|e| {
            log::error!("Failed to save config: {}", e);
            e.to_string()
        })?;

    log::info!("Timer configuration updated successfully");

    // Publish ConfigUpdated event
    let config_updated = domain::ConfigUpdated::new(config.clone());
    event_publisher.publish(Box::new(config_updated));

    Ok(config)
}
