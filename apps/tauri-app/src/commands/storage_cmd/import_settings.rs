use super::*;

#[command]
pub async fn import_settings(
    json_string: String,
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
) -> Result<(), String> {
    let config: Config = serde_json::from_str(&json_string)
        .context("Failed to deserialize configuration")
        .map_err(|e| e.to_string())?;

    config
        .validate()
        .context("Invalid configuration")
        .map_err(|e| e.to_string())?;

    config_repo
        .save_config(&config)
        .await
        .context("Failed to save imported configuration")
        .map_err(|e| e.to_string())?;

    Ok(())
}
