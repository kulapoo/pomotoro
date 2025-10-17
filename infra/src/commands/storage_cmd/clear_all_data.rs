use super::*;

#[command]
pub async fn clear_all_data(
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
) -> Result<(), String> {
    config_repo
        .reset_to_defaults()
        .await
        .context("Failed to clear all data")
        .map_err(|e| e.to_string())?;

    let config_dir = dirs::config_dir()
        .ok_or_else(|| "Could not determine config directory".to_string())?;

    let pomotoro_dir = config_dir.join("pomotoro");

    if pomotoro_dir.exists() {
        std::fs::remove_dir_all(&pomotoro_dir)
            .context("Failed to remove data directory")
            .map_err(|e| e.to_string())?;

        std::fs::create_dir_all(&pomotoro_dir)
            .context("Failed to recreate data directory")
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}