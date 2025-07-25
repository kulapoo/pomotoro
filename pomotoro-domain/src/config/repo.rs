use async_trait::async_trait;
use crate::Result;
use super::config::Config;

#[async_trait]
pub trait ConfigRepository: Send + Sync {
    async fn get_config(&self) -> Result<Config>;
    async fn save_config(&self, config: &Config) -> Result<()>;
    async fn reset_to_defaults(&self) -> Result<Config>;
    async fn config_exists(&self) -> Result<bool>;
}