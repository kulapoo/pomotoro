use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn save_global_config(
    config: Config,
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
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
    let config_updated = ConfigUpdated::new(config);
    event_publisher.publish(Box::new(config_updated));

    Ok(())
}
