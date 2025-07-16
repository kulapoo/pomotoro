use super::types::GlobalConfig;
use std::sync::{Arc, RwLock};
use std::path::PathBuf;
use serde_json;
use std::fs;
use std::io::{self, Write};
use tauri::{AppHandle, Manager};

pub type ConfigRepository = Arc<dyn ConfigRepositoryTrait + Send + Sync>;

pub trait ConfigRepositoryTrait {
    fn get_config(&self) -> Result<GlobalConfig, ConfigError>;
    fn save_config(&self, config: &GlobalConfig) -> Result<(), ConfigError>;
    fn reset_to_defaults(&self) -> Result<GlobalConfig, ConfigError>;
}

#[derive(Debug)]
pub enum ConfigError {
    IoError(io::Error),
    SerializationError(serde_json::Error),
    ConfigNotFound,
    InvalidConfig,
}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        ConfigError::IoError(error)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(error: serde_json::Error) -> Self {
        ConfigError::SerializationError(error)
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            ConfigError::ConfigNotFound => write!(f, "Configuration file not found"),
            ConfigError::InvalidConfig => write!(f, "Invalid configuration"),
        }
    }
}

impl std::error::Error for ConfigError {}

pub struct FileConfigRepository {
    config_path: PathBuf,
    cache: Arc<RwLock<Option<GlobalConfig>>>,
}

impl FileConfigRepository {
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

impl ConfigRepositoryTrait for FileConfigRepository {
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

pub struct InMemoryConfigRepository {
    config: Arc<RwLock<GlobalConfig>>,
}

impl InMemoryConfigRepository {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(GlobalConfig::default())),
        }
    }
}

impl ConfigRepositoryTrait for InMemoryConfigRepository {
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