use domain::{Config, ConfigRepository, Result};
use std::sync::Arc;

pub async fn get_config(
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
) -> Result<Config> {
    match config_repo.config_exists().await? {
        true => config_repo.get_config().await,
        false => {
            let default_config = Config::default();
            config_repo.save_config(&default_config).await?;
            Ok(default_config)
        }
    }
}
