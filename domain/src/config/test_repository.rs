use crate::{
    Result,
    config::{Config, ConfigRepository},
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

/// In-memory config repository for testing purposes
#[derive(Debug)]
pub struct InMemoryConfigRepository {
    config: Arc<Mutex<Option<Config>>>,
}

impl InMemoryConfigRepository {
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_config(config: Config) -> Self {
        Self {
            config: Arc::new(Mutex::new(Some(config))),
        }
    }
}

impl Default for InMemoryConfigRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConfigRepository for InMemoryConfigRepository {
    async fn get_config(&self) -> Result<Config> {
        let config = self.config.lock().unwrap();
        match config.as_ref() {
            Some(cfg) => Ok(cfg.clone()),
            None => Ok(Config::default()),
        }
    }

    async fn save_config(&self, config: &Config) -> Result<()> {
        let mut stored_config = self.config.lock().unwrap();
        *stored_config = Some(config.clone());
        Ok(())
    }

    async fn reset_to_defaults(&self) -> Result<Config> {
        let default_config = Config::default();
        let mut stored_config = self.config.lock().unwrap();
        *stored_config = Some(default_config.clone());
        Ok(default_config)
    }

    async fn config_exists(&self) -> Result<bool> {
        let config = self.config.lock().unwrap();
        Ok(config.is_some())
    }
}
