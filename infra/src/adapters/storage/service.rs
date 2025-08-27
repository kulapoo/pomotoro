use super::storage_config::{StorageConfig, StorageLocation};
use chrono::{DateTime, Utc};
use domain::{Error, Result as DomainResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub version: String,
    pub exported_at: DateTime<Utc>,
    pub tasks: serde_json::Value,
    pub timer_state: serde_json::Value,
    pub configuration: serde_json::Value,
    pub session_history: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub path: PathBuf,
    pub size_bytes: u64,
}

pub struct FileStorageService {
    config: Arc<RwLock<StorageConfig>>,
    base_path: Arc<RwLock<PathBuf>>,
}

impl FileStorageService {
    pub fn new(config: StorageConfig) -> DomainResult<Self> {
        let base_path = config.storage_location.ensure_exists()
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to create storage directory: {e}"),
            })?;
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            base_path: Arc::new(RwLock::new(base_path)),
        })
    }

    pub async fn get_storage_path(&self) -> PathBuf {
        self.base_path.read().await.clone()
    }

    pub async fn set_storage_location(&self, location: StorageLocation) -> DomainResult<()> {
        let new_path = location.ensure_exists()
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to create storage directory: {e}"),
            })?;
        
        let old_path = self.base_path.read().await.clone();
        
        if old_path != new_path {
            self.migrate_data(&old_path, &new_path).await?;
        }
        
        let mut config = self.config.write().await;
        config.storage_location = location;
        
        let mut base_path = self.base_path.write().await;
        *base_path = new_path;
        
        Ok(())
    }

    pub async fn export_data(&self) -> DomainResult<ExportData> {
        let base_path = self.base_path.read().await;
        
        let tasks_file = base_path.join("tasks.json");
        let tasks_data = if tasks_file.exists() {
            let content = tokio::fs::read_to_string(&tasks_file)
                .await
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to read tasks file: {e}"),
                })?;
            serde_json::from_str(&content)
                .unwrap_or(serde_json::Value::Null)
        } else {
            serde_json::Value::Array(vec![])
        };
        
        let timer_file = base_path.join("timer_data.json");
        let timer_data = if timer_file.exists() {
            let content = tokio::fs::read_to_string(&timer_file)
                .await
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to read timer file: {e}"),
                })?;
            serde_json::from_str(&content)
                .unwrap_or(serde_json::Value::Null)
        } else {
            serde_json::Value::Null
        };
        
        let config_file = base_path.join("Config.json");
        let config_data = if config_file.exists() {
            let content = tokio::fs::read_to_string(&config_file)
                .await
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to read config file: {e}"),
                })?;
            serde_json::from_str(&content)
                .unwrap_or(serde_json::Value::Null)
        } else {
            serde_json::Value::Null
        };
        
        Ok(ExportData {
            version: "1.0.0".to_string(),
            exported_at: Utc::now(),
            tasks: tasks_data,
            timer_state: timer_data,
            configuration: config_data,
            session_history: None,
        })
    }

    pub async fn import_data(&self, data: ExportData) -> DomainResult<()> {
        let base_path = self.base_path.read().await;
        
        if !data.tasks.is_null() {
            let tasks_file = base_path.join("tasks.json");
            let content = serde_json::to_string_pretty(&data.tasks)
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to serialize tasks: {e}"),
                })?;
            tokio::fs::write(&tasks_file, content)
                .await
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to write tasks file: {e}"),
                })?;
        }
        
        if !data.timer_state.is_null() {
            let timer_file = base_path.join("timer_data.json");
            let content = serde_json::to_string_pretty(&data.timer_state)
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to serialize timer state: {e}"),
                })?;
            tokio::fs::write(&timer_file, content)
                .await
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to write timer file: {e}"),
                })?;
        }
        
        if !data.configuration.is_null() {
            let config_file = base_path.join("Config.json");
            let content = serde_json::to_string_pretty(&data.configuration)
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to serialize configuration: {e}"),
                })?;
            tokio::fs::write(&config_file, content)
                .await
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to write config file: {e}"),
                })?;
        }
        
        Ok(())
    }

    pub async fn create_backup(&self) -> DomainResult<BackupInfo> {
        let config = self.config.read().await;
        if !config.backup_enabled {
            return Err(Error::ConfigurationError {
                message: "Backups are disabled".to_string(),
            });
        }
        
        let base_path = self.base_path.read().await;
        let backup_dir = base_path.join("backups");
        tokio::fs::create_dir_all(&backup_dir)
            .await
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to create backup directory: {e}"),
            })?;
        
        let backup_id = format!("{}", Utc::now().timestamp());
        let backup_path = backup_dir.join(format!("backup_{}.json", backup_id));
        
        let export_data = self.export_data().await?;
        let content = serde_json::to_string_pretty(&export_data)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to serialize backup data: {e}"),
            })?;
        
        tokio::fs::write(&backup_path, &content)
            .await
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to write backup file: {e}"),
            })?;
        
        self.cleanup_old_backups(&backup_dir, config.max_backup_count).await?;
        
        Ok(BackupInfo {
            id: backup_id,
            created_at: Utc::now(),
            path: backup_path,
            size_bytes: content.len() as u64,
        })
    }

    pub async fn restore_backup(&self, backup_id: String) -> DomainResult<()> {
        let base_path = self.base_path.read().await;
        let backup_path = base_path.join("backups").join(format!("backup_{}.json", backup_id));
        
        if !backup_path.exists() {
            return Err(Error::RepositoryError {
                message: format!("Backup not found: {}", backup_id),
            });
        }
        
        let content = tokio::fs::read_to_string(&backup_path)
            .await
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to read backup file: {e}"),
            })?;
        
        let data: ExportData = serde_json::from_str(&content)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to deserialize backup data: {e}"),
            })?;
        
        self.import_data(data).await?;
        
        Ok(())
    }

    async fn migrate_data(&self, old_path: &PathBuf, new_path: &PathBuf) -> DomainResult<()> {
        let files = vec!["tasks.json", "timer_data.json", "Config.json"];
        
        for file_name in files {
            let old_file = old_path.join(file_name);
            let new_file = new_path.join(file_name);
            
            if old_file.exists() {
                tokio::fs::copy(&old_file, &new_file)
                    .await
                    .map_err(|e| Error::RepositoryError {
                        message: format!("Failed to migrate file {}: {}", file_name, e),
                    })?;
            }
        }
        
        let old_backup_dir = old_path.join("backups");
        if old_backup_dir.exists() {
            let new_backup_dir = new_path.join("backups");
            tokio::fs::create_dir_all(&new_backup_dir)
                .await
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to create backup directory: {e}"),
                })?;
            
            let mut entries = tokio::fs::read_dir(&old_backup_dir)
                .await
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to read backup directory: {e}"),
                })?;
            
            while let Some(entry) = entries.next_entry().await
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to read backup entry: {e}"),
                })? {
                let old_backup = entry.path();
                if let Some(file_name) = old_backup.file_name() {
                    let new_backup = new_backup_dir.join(file_name);
                    tokio::fs::copy(&old_backup, &new_backup)
                        .await
                        .map_err(|e| Error::RepositoryError {
                            message: format!("Failed to migrate backup: {e}"),
                        })?;
                }
            }
        }
        
        Ok(())
    }

    async fn cleanup_old_backups(&self, backup_dir: &PathBuf, max_count: usize) -> DomainResult<()> {
        let mut entries = tokio::fs::read_dir(backup_dir)
            .await
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to read backup directory: {e}"),
            })?;
        
        let mut backups = Vec::new();
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to read backup entry: {e}"),
            })? {
            if let Ok(metadata) = entry.metadata().await {
                if metadata.is_file() {
                    backups.push(entry.path());
                }
            }
        }
        
        if backups.len() > max_count {
            backups.sort();
            let to_remove = backups.len() - max_count;
            for path in backups.iter().take(to_remove) {
                tokio::fs::remove_file(path).await.ok();
            }
        }
        
        Ok(())
    }
}