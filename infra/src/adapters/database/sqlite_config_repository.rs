use super::{DbPool, models::ConfigDb};
use crate::schema::config;
use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use domain::{Config, ConfigRepository, Error, Result};
use std::sync::Arc;

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
        log::debug!("Repository: Getting config from database");

        let mut conn = self.pool.get().map_err(|e| {
            let err = Error::RepositoryError {
                message: format!("Failed to get connection: {}", e),
            };
            log::error!("Repository: {}", err);
            err
        })?;

        // Check if config exists
        let count = config::table
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| {
                let err = Error::RepositoryError {
                    message: format!("Failed to count config rows: {}", e),
                };
                log::error!("Repository: {}", err);
                err
            })?;

        if count == 0 {
            log::info!("Repository: No config found, creating default config");

            let default_config = Config::default();
            let config_json =
                serde_json::to_string(&default_config).map_err(|e| {
                    let err = Error::RepositoryError {
                        message: format!(
                            "Failed to serialize default config: {}",
                            e
                        ),
                    };
                    log::error!("Repository: {}", err);
                    err
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
                .map_err(|e| {
                    let err = Error::RepositoryError {
                        message: format!("Failed to insert default config: {}", e),
                    };
                    log::error!("Repository: {}", err);
                    err
                })?;

            log::info!("Repository: Default config created successfully");
        }

        let config_db = config::table
            .filter(config::id.eq(1))
            .first::<ConfigDb>(&mut conn)
            .map_err(|e| {
                let err = Error::RepositoryError {
                    message: format!("Failed to get config: {}", e),
                };
                log::error!("Repository: {}", err);
                err
            })?;

        let config =
            serde_json::from_str(&config_db.config_data).map_err(|e| {
                let err = Error::RepositoryError {
                    message: format!("Failed to deserialize config: {}", e),
                };
                log::error!("Repository: {}", err);
                err
            })?;

        log::debug!("Repository: Config retrieved successfully");
        Ok(config)
    }

    async fn save_config(&self, config: &Config) -> Result<()> {
        log::debug!("Repository: Saving config to database");

        let config = config.clone();
        let config_json = serde_json::to_string(&config).map_err(|e| {
            let err = Error::RepositoryError {
                message: format!("Failed to serialize config: {}", e),
            };
            log::error!("Repository: {}", err);
            err
        })?;

        let mut conn = self.pool.get().map_err(|e| {
            let err = Error::RepositoryError {
                message: format!("Failed to get connection: {}", e),
            };
            log::error!("Repository: {}", err);
            err
        })?;

        // Ensure config exists
        let count = config::table
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| {
                let err = Error::RepositoryError {
                    message: format!("Failed to count config rows: {}", e),
                };
                log::error!("Repository: {}", err);
                err
            })?;

        if count == 0 {
            log::info!("Repository: Inserting new config");

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
                .map_err(|e| {
                    let err = Error::RepositoryError {
                        message: format!("Failed to insert config: {}", e),
                    };
                    log::error!("Repository: {}", err);
                    err
                })?;

            log::info!("Repository: Config inserted successfully");
        } else {
            log::debug!("Repository: Updating existing config");

            diesel::update(config::table.filter(config::id.eq(1)))
                .set((
                    config::config_data.eq(config_json),
                    config::updated_at.eq(Utc::now().to_rfc3339()),
                ))
                .execute(&mut conn)
                .map_err(|e| {
                    let err = Error::RepositoryError {
                        message: format!("Failed to update config: {}", e),
                    };
                    log::error!("Repository: {}", err);
                    err
                })?;

            log::info!("Repository: Config updated successfully");
        }

        Ok(())
    }

    async fn reset_to_defaults(&self) -> Result<Config> {
        log::info!("Repository: Resetting config to defaults");

        let default_config = Config::default();
        self.save_config(&default_config).await?;

        log::info!("Repository: Config reset to defaults successfully");
        Ok(default_config)
    }

    async fn config_exists(&self) -> Result<bool> {
        log::debug!("Repository: Checking if config exists");

        let mut conn = self.pool.get().map_err(|e| {
            let err = Error::RepositoryError {
                message: format!("Failed to get connection: {}", e),
            };
            log::error!("Repository: {}", err);
            err
        })?;

        let count = config::table
            .count()
            .get_result::<i64>(&mut conn)
            .map_err(|e| {
                let err = Error::RepositoryError {
                    message: format!("Failed to count config rows: {}", e),
                };
                log::error!("Repository: {}", err);
                err
            })?;

        let exists = count > 0;
        log::debug!("Repository: Config exists: {}", exists);
        Ok(exists)
    }
}
