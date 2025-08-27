use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub storage_location: StorageLocation,
    pub auto_save_interval_seconds: u32,
    pub backup_enabled: bool,
    pub max_backup_count: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            storage_location: StorageLocation::default(),
            auto_save_interval_seconds: 60,
            backup_enabled: true,
            max_backup_count: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageLocation {
    Default,
    Custom(PathBuf),
}

impl Default for StorageLocation {
    fn default() -> Self {
        Self::Default
    }
}

impl StorageLocation {
    pub fn get_path(&self) -> PathBuf {
        match self {
            StorageLocation::Default => {
                dirs::data_dir()
                    .expect("Failed to get user data directory")
                    .join("pomotoro")
            }
            StorageLocation::Custom(path) => path.clone(),
        }
    }

    pub fn ensure_exists(&self) -> std::io::Result<PathBuf> {
        let path = self.get_path();
        std::fs::create_dir_all(&path)?;
        Ok(path)
    }
}