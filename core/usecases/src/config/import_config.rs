use domain::{Config, ConfigRepository, EventPublisher, Result};
use std::sync::Arc;

pub async fn import_config(
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
    _event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    json_string: &str,
) -> Result<Config> {
    let config: Config = serde_json::from_str(json_string).map_err(|e| {
        domain::Error::DeserializationError {
            message: e.to_string(),
        }
    })?;

    config.validate()?;

    config_repo.save_config(&config).await?;

    Ok(config)
}

pub async fn import_config_from_file(
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    file_path: &str,
) -> Result<Config> {
    let json_string = std::fs::read_to_string(file_path).map_err(|e| {
        domain::Error::IoError {
            message: e.to_string(),
        }
    })?;

    import_config(config_repo, event_publisher, &json_string).await
}
