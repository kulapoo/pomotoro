use domain::{ConfigRepository, Result};
use std::sync::Arc;

pub async fn export_config(
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
) -> Result<String> {
    let config = config_repo.get_config().await?;
    let json = serde_json::to_string_pretty(&config).map_err(|e| {
        domain::Error::SerializationError {
            message: e.to_string(),
        }
    })?;
    Ok(json)
}

pub async fn export_config_to_file(
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
    file_path: &str,
) -> Result<()> {
    let config = config_repo.get_config().await?;
    let json = serde_json::to_string_pretty(&config).map_err(|e| {
        domain::Error::SerializationError {
            message: e.to_string(),
        }
    })?;

    std::fs::write(file_path, json).map_err(|e| domain::Error::IoError {
        message: e.to_string(),
    })?;

    Ok(())
}
