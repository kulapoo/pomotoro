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
    
    fn ensure_config_exists(&self) -> Result<()> {
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;
        
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
        
        Ok(())
    }
}

#[async_trait]
impl ConfigRepository for SqliteConfigRepository {
    async fn get_config(&self) -> Result<Config> {
        self.ensure_config_exists()?;
        
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;
        
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
    }
    
    async fn save_config(&self, config: &Config) -> Result<()> {
        self.ensure_config_exists()?;
        
        let config_json = serde_json::to_string(config)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to serialize config: {}", e),
            })?;
        
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;
        
        diesel::update(config::table.filter(config::id.eq(1)))
            .set((
                config::config_data.eq(config_json),
                config::updated_at.eq(Utc::now().to_rfc3339()),
            ))
            .execute(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to update config: {}", e),
            })?;
        
        Ok(())
    }
    
    async fn reset_to_defaults(&self) -> Result<Config> {
        let default_config = Config::default();
        self.save_config(&default_config).await?;
        Ok(default_config)
    }
    
    async fn config_exists(&self) -> Result<bool> {
        let mut conn = self.pool.get().map_err(|e| Error::RepositoryError {
            message: format!("Failed to get connection: {}", e),
        })?;
        
        let count = config::table
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| Error::RepositoryError {
                message: format!("Failed to count config rows: {}", e),
            })?;
        
        Ok(count > 0)
    }
}