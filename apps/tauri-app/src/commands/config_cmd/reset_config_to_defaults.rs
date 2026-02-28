use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn reset_config_to_defaults(
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
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
    let config_reset = ConfigReset::new(config.clone());
    event_publisher.publish(Box::new(config_reset));

    Ok(config)
}
