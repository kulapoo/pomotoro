use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn update_general_config(
    preferences: GeneralConfig,
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
    event_publisher: State<'_, EventPublisherArc>,
) -> Result<Config, String> {
    log::info!(
        "Received update_general_config command with preferences: {:?}",
        preferences
    );
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
    let config_updated = ConfigUpdated::new(config.clone());
    event_publisher.publish(Box::new(config_updated));

    Ok(config)
}