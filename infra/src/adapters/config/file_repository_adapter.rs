use async_trait::async_trait;
use domain::{Config, ConfigRepository, Result, Error};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct FileConfigRepository {
    config_path: PathBuf,
    cache: Arc<RwLock<Option<Config>>>,
}

impl FileConfigRepository {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    async fn load_from_file(&self) -> Result<Config> {
        if !self.config_path.exists() {
            let default_config = Config::default();
            self.save_to_file(&default_config).await?;
            return Ok(default_config);
        }

        let content = tokio::fs::read_to_string(&self.config_path)
            .await
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to read config file: {e}"),
            })?;

        let config_dto: super::config_dto::ConfigDto = serde_json::from_str(&content)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to parse config file: {e}"),
            })?;

        Config::try_from(config_dto)
    }

    async fn save_to_file(&self, config: &Config) -> Result<()> {
        let config_dto = super::config_dto::ConfigDto::from(config.clone());
        let content = serde_json::to_string_pretty(&config_dto)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to serialize config: {e}"),
            })?;
        
        let temp_file = self.config_path.with_extension("tmp");
        
        tokio::fs::write(&temp_file, content)
            .await
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to write config file: {e}"),
            })?;
        
        tokio::fs::rename(&temp_file, &self.config_path)
            .await
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to rename config file: {e}"),
            })?;
        
        Ok(())
    }

    async fn update_cache(&self, config: &Config) {
        let mut cache = self.cache.write().await;
        *cache = Some(config.clone());
    }

    async fn get_cached(&self) -> Option<Config> {
        let cache = self.cache.read().await;
        cache.clone()
    }
}

#[async_trait]
impl ConfigRepository for FileConfigRepository {
    async fn get_config(&self) -> Result<Config> {
        if let Some(cached_config) = self.get_cached().await {
            return Ok(cached_config);
        }

        let config = self.load_from_file().await?;
        self.update_cache(&config).await;
        Ok(config)
    }

    async fn save_config(&self, config: &Config) -> Result<()> {
        config.validate()?;
        self.save_to_file(config).await?;
        self.update_cache(config).await;
        Ok(())
    }

    async fn reset_to_defaults(&self) -> Result<Config> {
        let default_config = Config::default();
        self.save_config(&default_config).await?;
        Ok(default_config)
    }

    async fn config_exists(&self) -> Result<bool> {
        Ok(self.config_path.exists())
    }
}