use super::models::GlobalConfig;
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use serde_json;
use std::fs;
use std::io::{self, Write};
use tauri::{AppHandle, Manager};
use thiserror::Error;

pub type ConfigRepository = Arc<dyn ConfigRepo + Send + Sync>;

pub trait ConfigRepo {
    fn get_config(&self) -> Result<GlobalConfig, ConfigError>;
    fn save_config(&self, config: &GlobalConfig) -> Result<(), ConfigError>;
    fn reset_to_defaults(&self) -> Result<GlobalConfig, ConfigError>;
}

pub struct FileConfigRepo {
    config_path: PathBuf,
    cache: Arc<RwLock<Option<GlobalConfig>>>,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Configuration file not found")]
    ConfigNotFound,
    #[error("Invalid configuration")]
    InvalidConfig,
}

impl FileConfigRepo {
    pub fn new(app_handle: &AppHandle) -> Result<Self, ConfigError> {
        let config_dir = app_handle
            .path()
            .app_config_dir()
            .map_err(|_| ConfigError::InvalidConfig)?;

        fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("config.json");

        Ok(Self {
            config_path,
            cache: Arc::new(RwLock::new(None)),
        })
    }


    fn load_from_file(&self) -> Result<GlobalConfig, ConfigError> {
        if !self.config_path.exists() {
            return Err(ConfigError::ConfigNotFound);
        }

        let content = fs::read_to_string(&self.config_path)?;
        let config: GlobalConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    fn save_to_file(&self, config: &GlobalConfig) -> Result<(), ConfigError> {
        let content = serde_json::to_string_pretty(config)?;
        let mut file = fs::File::create(&self.config_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        Ok(())
    }

    fn update_cache(&self, config: &GlobalConfig) {
        if let Ok(mut cache) = self.cache.write() {
            *cache = Some(config.clone());
        }
    }

    fn get_cached(&self) -> Option<GlobalConfig> {
        self.cache.read().ok()?.clone()
    }
}

impl ConfigRepo for FileConfigRepo {
    fn get_config(&self) -> Result<GlobalConfig, ConfigError> {
        if let Some(cached_config) = self.get_cached() {
            return Ok(cached_config);
        }

        let config = match self.load_from_file() {
            Ok(config) => config,
            Err(ConfigError::ConfigNotFound) => {
                let default_config = GlobalConfig::default();
                self.save_to_file(&default_config)?;
                default_config
            }
            Err(e) => return Err(e),
        };

        self.update_cache(&config);
        Ok(config)
    }

    fn save_config(&self, config: &GlobalConfig) -> Result<(), ConfigError> {
        self.save_to_file(config)?;
        self.update_cache(config);
        Ok(())
    }

    fn reset_to_defaults(&self) -> Result<GlobalConfig, ConfigError> {
        let default_config = GlobalConfig::default();
        self.save_config(&default_config)?;
        Ok(default_config)
    }
}

pub struct InMemoryConfigRepo {
    config: Arc<RwLock<GlobalConfig>>,
}

impl InMemoryConfigRepo {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(GlobalConfig::default())),
        }
    }
}

impl ConfigRepo for InMemoryConfigRepo {
    fn get_config(&self) -> Result<GlobalConfig, ConfigError> {
        self.config
            .read()
            .map(|config| config.clone())
            .map_err(|_| ConfigError::InvalidConfig)
    }

    fn save_config(&self, config: &GlobalConfig) -> Result<(), ConfigError> {
        self.config
            .write()
            .map(|mut stored_config| *stored_config = config.clone())
            .map_err(|_| ConfigError::InvalidConfig)
    }

    fn reset_to_defaults(&self) -> Result<GlobalConfig, ConfigError> {
        let default_config = GlobalConfig::default();
        self.save_config(&default_config)?;
        Ok(default_config)
    }
}