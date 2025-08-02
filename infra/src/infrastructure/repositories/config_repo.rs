use async_trait::async_trait;
use domain::{ConfigRepository, Config, Result};
use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct InMemoryConfigRepository {
    config: Arc<RwLock<Option<Config>>>,
}

impl InMemoryConfigRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: Config) -> Self {
        Self {
            config: Arc::new(RwLock::new(Some(config))),
        }
    }
}

#[async_trait]
impl ConfigRepository for InMemoryConfigRepository {
    async fn get_config(&self) -> Result<Config> {
        let config = self.config.read().unwrap();
        Ok(config.clone().unwrap_or_default())
    }

    async fn save_config(&self, config: &Config) -> Result<()> {
        config.validate()?;
        let mut stored_config = self.config.write().unwrap();
        *stored_config = Some(config.clone());
        Ok(())
    }

    async fn reset_to_defaults(&self) -> Result<Config> {
        let default_config = Config::default();
        let mut stored_config = self.config.write().unwrap();
        *stored_config = Some(default_config.clone());
        Ok(default_config)
    }

    async fn config_exists(&self) -> Result<bool> {
        let config = self.config.read().unwrap();
        Ok(config.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_save_and_retrieve_config() {
        let repo = InMemoryConfigRepository::new();
        let mut config = Config::default();
        config.general.minimize_to_tray = false;

        repo.save_config(&config).await.unwrap();
        let retrieved = repo.get_config().await.unwrap();

        assert!(!retrieved.general.minimize_to_tray);
    }

    #[tokio::test]
    async fn should_reset_to_defaults() {
        let repo = InMemoryConfigRepository::new();
        let mut config = Config::default();
        config.general.start_minimized = true;
        
        repo.save_config(&config).await.unwrap();
        let reset_config = repo.reset_to_defaults().await.unwrap();

        assert!(reset_config.general.auto_start_breaks); // Default value
    }

    #[tokio::test]
    async fn should_validate_config_before_saving() {
        let repo = InMemoryConfigRepository::new();
        let mut config = Config::default();
        // GeneralConfig doesn't have validation that would fail, so use a different approach
        // Let's test with audio config validation instead
        config.audio.volume = 2.0; // Invalid value > 1.0

        let result = repo.save_config(&config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn should_check_config_existence() {
        let repo = InMemoryConfigRepository::new();
        
        assert!(!repo.config_exists().await.unwrap());
        
        repo.save_config(&Config::default()).await.unwrap();
        
        assert!(repo.config_exists().await.unwrap());
    }
}