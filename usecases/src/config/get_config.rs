use domain::{Config, ConfigRepository, Result};
use std::sync::Arc;

pub async fn get_config(config_repo: &Arc<dyn ConfigRepository + Send + Sync>) -> Result<Config> {
    match config_repo.config_exists().await? {
        true => config_repo.get_config().await,
        false => {
            let default_config = Config::default();
            config_repo.save_config(&default_config).await?;
            Ok(default_config)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use domain::InMemoryConfigRepository;

    #[tokio::test]
    async fn should_get_existing_config() {
        let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
            Arc::new(InMemoryConfigRepository::new());
        let mut config = Config::default();
        config.general.auto_start_breaks = false;

        config_repo.save_config(&config).await.unwrap();

        let retrieved_config = get_config(&config_repo).await.unwrap();

        assert!(!retrieved_config.general.auto_start_breaks);
    }

    #[tokio::test]
    async fn should_get_default_config_when_none_exists() {
        let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
            Arc::new(InMemoryConfigRepository::new());

        let config = get_config(&config_repo).await.unwrap();

        assert!(config.general.auto_start_breaks);
    }

    #[tokio::test]
    async fn should_get_config_or_create_default() {
        let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
            Arc::new(InMemoryConfigRepository::new());

        assert!(!config_repo.config_exists().await.unwrap());

        let config = get_config(&config_repo).await.unwrap();

        assert!(config.general.auto_start_breaks);

        assert!(config_repo.config_exists().await.unwrap());
    }

    #[tokio::test]
    async fn should_return_existing_config_when_available() {
        let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
            Arc::new(InMemoryConfigRepository::new());
        let mut custom_config = Config::default();
        custom_config.general.start_minimized = true;

        config_repo.save_config(&custom_config).await.unwrap();

        let config = get_config(&config_repo).await.unwrap();

        assert!(config.general.start_minimized);
    }

    #[tokio::test]
    async fn should_check_config_existence() {
        let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
            Arc::new(InMemoryConfigRepository::new());

        assert!(!config_repo.config_exists().await.unwrap());

        config_repo.save_config(&Config::default()).await.unwrap();

        assert!(config_repo.config_exists().await.unwrap());
    }
}
