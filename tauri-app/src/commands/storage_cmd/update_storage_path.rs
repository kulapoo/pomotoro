use super::*;

#[command]
pub async fn update_storage_path(
    new_path: String,
    _config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
) -> Result<(), String> {
    let new_path = PathBuf::from(new_path);

    if !new_path.is_absolute() {
        return Err("Path must be absolute".to_string());
    }

    if !new_path.exists() {
        std::fs::create_dir_all(&new_path)
            .context("Failed to create new storage directory")
            .map_err(|e| e.to_string())?;
    }

    let old_config_dir = dirs::config_dir()
        .ok_or_else(|| "Could not determine config directory".to_string())?
        .join("pomotoro");

    if old_config_dir.exists() && old_config_dir != new_path {
        for entry in std::fs::read_dir(&old_config_dir)
            .context("Failed to read old data directory")
            .map_err(|e| e.to_string())?
        {
            let entry = entry.map_err(|e| e.to_string())?;
            let file_name = entry.file_name();
            let dest = new_path.join(file_name);

            std::fs::rename(entry.path(), dest)
                .context("Failed to move data files")
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}