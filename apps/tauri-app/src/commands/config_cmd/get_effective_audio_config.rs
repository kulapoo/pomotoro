use super::*;

#[tauri::command(rename_all = "snake_case")]
pub async fn get_effective_audio_config(
    _task_id: TaskId,
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
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
