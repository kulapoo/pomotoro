use super::*;

#[command]
pub async fn open_data_directory() -> Result<(), String> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| "Could not determine config directory".to_string())?;

    let pomotoro_dir = config_dir.join("pomotoro");

    if !pomotoro_dir.exists() {
        std::fs::create_dir_all(&pomotoro_dir)
            .context("Failed to create data directory")
            .map_err(|e| e.to_string())?;
    }

    open::that(pomotoro_dir)
        .context("Failed to open data directory")
        .map_err(|e| e.to_string())?;

    Ok(())
}
