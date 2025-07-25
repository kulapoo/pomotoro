use pomotoro_domain::{Config, ConfigRepository, Result};
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
    use crate::infrastructure::InMemoryConfigRepository;

    #[tokio::test]
    async fn should_get_existing_config() {
        let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
            Arc::new(InMemoryConfigRepository::new());
        let mut config = Config::default();
        config.general.max_sessions_default = 6;

        config_repo.save_config(&config).await.unwrap();

        let retrieved_config = get_config(&config_repo).await.unwrap();

        assert_eq!(retrieved_config.general.max_sessions_default, 6);
    }

    #[tokio::test]
    async fn should_get_default_config_when_none_exists() {
        let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
            Arc::new(InMemoryConfigRepository::new());

        let config = get_config(&config_repo).await.unwrap();

        // Should return default config
        assert_eq!(config.general.max_sessions_default, 4); // Default value
    }

    #[tokio::test]
    async fn should_get_config_or_create_default() {
        let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
            Arc::new(InMemoryConfigRepository::new());

        // Initially no config exists
        assert!(!config_repo.config_exists().await.unwrap());

        let config = get_config(&config_repo).await.unwrap();

        // Should return default config and save it
        assert_eq!(config.general.max_sessions_default, 4);

        // Config should now exist
        assert!(config_repo.config_exists().await.unwrap());
    }

    #[tokio::test]
    async fn should_return_existing_config_when_available() {
        let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
            Arc::new(InMemoryConfigRepository::new());
        let mut custom_config = Config::default();
        custom_config.general.max_sessions_default = 8;

        config_repo.save_config(&custom_config).await.unwrap();

        let config = get_config(&config_repo).await.unwrap();

        // Should return existing custom config, not default
        assert_eq!(config.general.max_sessions_default, 8);
    }

    #[tokio::test]
    async fn should_check_config_existence() {
        let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
            Arc::new(InMemoryConfigRepository::new());

        // Initially no config
        assert!(!config_repo.config_exists().await.unwrap());

        // Save a config
        config_repo.save_config(&Config::default()).await.unwrap();

        // Now config exists
        assert!(config_repo.config_exists().await.unwrap());
    }
}
