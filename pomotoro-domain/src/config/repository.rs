use async_trait::async_trait;
use crate::{Config, Result};
use std::sync::{Arc, RwLock};

#[async_trait]
pub trait ConfigRepository: Send + Sync {
    async fn get_config(&self) -> Result<Config>;
    async fn save_config(&self, config: &Config) -> Result<()>;
    async fn reset_to_defaults(&self) -> Result<Config>;
    async fn config_exists(&self) -> Result<bool>;
}

// Test implementation for ConfigRepository - available for use in tests
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
        config.general.max_sessions_default = 6;

        repo.save_config(&config).await.unwrap();
        let retrieved = repo.get_config().await.unwrap();

        assert_eq!(retrieved.general.max_sessions_default, 6);
    }

    #[tokio::test]
    async fn should_reset_to_defaults() {
        let repo = InMemoryConfigRepository::new();
        let mut config = Config::default();
        config.general.max_sessions_default = 8;
        
        repo.save_config(&config).await.unwrap();
        let reset_config = repo.reset_to_defaults().await.unwrap();

        assert_eq!(reset_config.general.max_sessions_default, 4); // Default value
    }

    #[tokio::test]
    async fn should_validate_config_before_saving() {
        let repo = InMemoryConfigRepository::new();
        let mut config = Config::default();
        config.general.max_sessions_default = 0; // Invalid value

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