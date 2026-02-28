use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn update_audio_config(
    _task_id: TaskId,
    audio_config: AudioConfig,
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
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
    let config_updated = ConfigUpdated::new(config.clone());
    event_publisher.publish(Box::new(config_updated));

    Ok(config)
}
