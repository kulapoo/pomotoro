use anyhow::Result;
use domain::ConfigRepository;
use tauri::{State, command};
use std::path::PathBuf;
use anyhow::Context;
use std::sync::Arc;

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

#[command]
pub async fn clear_all_data(
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
) -> Result<(), String> {
    config_repo.reset_to_defaults()
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

#[command]
pub async fn validate_storage_path(path: String) -> Result<bool, String> {
    let path = PathBuf::from(path);

    if !path.is_absolute() {
        return Ok(false);
    }

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            return Ok(false);
        }
    }

    Ok(true)
}

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

#[command]
pub async fn export_settings(
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
) -> Result<String, String> {
    let config = config_repo.get_config()
        .await
        .context("Failed to get current configuration")
        .map_err(|e| e.to_string())?;

    let json = serde_json::to_string_pretty(&config)
        .context("Failed to serialize configuration")
        .map_err(|e| e.to_string())?;

    Ok(json)
}

#[command]
pub async fn import_settings(
    json_string: String,
    config_repo: State<'_, Arc<dyn ConfigRepository + Send + Sync>>,
) -> Result<(), String> {
    let config: domain::Config = serde_json::from_str(&json_string)
        .context("Failed to deserialize configuration")
        .map_err(|e| e.to_string())?;

    config.validate()
        .context("Invalid configuration")
        .map_err(|e| e.to_string())?;

    config_repo.save_config(&config)
        .await
        .context("Failed to save imported configuration")
        .map_err(|e| e.to_string())?;

    Ok(())
}