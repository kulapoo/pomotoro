use async_trait::async_trait;
use diesel::prelude::*;
use domain::{
    ConfigRepository, Config, Error, Result,
};
use std::sync::Arc;
use chrono::Utc;
use crate::schema::config;
use super::{DbPool, models::ConfigDb};

pub struct SqliteConfigRepository {
    pool: Arc<DbPool>,
}

impl SqliteConfigRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConfigRepository for SqliteConfigRepository {
    async fn get_config(&self) -> Result<Config> {
        let pool = self.pool.clone();
        
        tauri::async_runtime::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| Error::RepositoryError {
                message: format!("Failed to get connection: {}", e),
            })?;
            
            // Check if config exists
            let count = config::table
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to count config rows: {}", e),
                })?;
            
            if count == 0 {
                let default_config = Config::default();
                let config_json = serde_json::to_string(&default_config)
                    .map_err(|e| Error::RepositoryError {
                        message: format!("Failed to serialize default config: {}", e),
                    })?;
                
                let now = Utc::now().to_rfc3339();
                let config_db = ConfigDb {
                    id: 1,
                    config_data: config_json,
                    created_at: now.clone(),
                    updated_at: now,
                };
                
                diesel::insert_into(config::table)
                    .values(&config_db)
                    .execute(&mut conn)
                    .map_err(|e| Error::RepositoryError {
                        message: format!("Failed to insert default config: {}", e),
                    })?;
            }
            
            let config_db = config::table
                .filter(config::id.eq(1))
                .first::<ConfigDb>(&mut conn)
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to get config: {}", e),
                })?;
            
            let config = serde_json::from_str(&config_db.config_data)
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to deserialize config: {}", e),
                })?;
            
            Ok(config)
        })
        .await
        .map_err(|e| Error::RepositoryError {
            message: format!("Task join error: {}", e),
        })?
    }
    
    async fn save_config(&self, config: &Config) -> Result<()> {
        let pool = self.pool.clone();
        let config = config.clone();
        
        tauri::async_runtime::spawn_blocking(move || {
            let config_json = serde_json::to_string(&config)
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to serialize config: {}", e),
                })?;
            
            let mut conn = pool.get().map_err(|e| Error::RepositoryError {
                message: format!("Failed to get connection: {}", e),
            })?;
            
            // Ensure config exists
            let count = config::table
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to count config rows: {}", e),
                })?;
            
            if count == 0 {
                let now = Utc::now().to_rfc3339();
                let config_db = ConfigDb {
                    id: 1,
                    config_data: config_json,
                    created_at: now.clone(),
                    updated_at: now,
                };
                
                diesel::insert_into(config::table)
                    .values(&config_db)
                    .execute(&mut conn)
                    .map_err(|e| Error::RepositoryError {
                        message: format!("Failed to insert config: {}", e),
                    })?;
            } else {
                diesel::update(config::table.filter(config::id.eq(1)))
                    .set((
                        config::config_data.eq(config_json),
                        config::updated_at.eq(Utc::now().to_rfc3339()),
                    ))
                    .execute(&mut conn)
                    .map_err(|e| Error::RepositoryError {
                        message: format!("Failed to update config: {}", e),
                    })?;
            }
            
            Ok(())
        })
        .await
        .map_err(|e| Error::RepositoryError {
            message: format!("Task join error: {}", e),
        })?
    }
    
    async fn reset_to_defaults(&self) -> Result<Config> {
        let default_config = Config::default();
        self.save_config(&default_config).await?;
        Ok(default_config)
    }
    
    async fn config_exists(&self) -> Result<bool> {
        let pool = self.pool.clone();
        
        tauri::async_runtime::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| Error::RepositoryError {
                message: format!("Failed to get connection: {}", e),
            })?;
            
            let count = config::table
                .count()
                .get_result::<i64>(&mut conn)
                .map_err(|e| Error::RepositoryError {
                    message: format!("Failed to count config rows: {}", e),
                })?;
            
            Ok(count > 0)
        })
        .await
        .map_err(|e| Error::RepositoryError {
            message: format!("Task join error: {}", e),
        })?
    }
}