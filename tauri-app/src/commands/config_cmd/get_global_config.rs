use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_global_config(
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
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