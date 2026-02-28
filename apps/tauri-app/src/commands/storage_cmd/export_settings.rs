use super::*;

#[command]
pub async fn export_settings(
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
) -> Result<String, String> {
    let config = config_repo
        .get_config()
        .await
        .context("Failed to get current configuration")
        .map_err(|e| e.to_string())?;

    let json = serde_json::to_string_pretty(&config)
        .context("Failed to serialize configuration")
        .map_err(|e| e.to_string())?;

    Ok(json)
}
