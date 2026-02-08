use domain::{
    Config, ConfigRepository, EventPublisher, Result,
    config::events::ConfigReset,
};
use std::sync::Arc;

pub async fn reset_config(
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<Config> {
    let default_config = config_repo.reset_to_defaults().await?;

    // Publish ConfigReset event
    let event = ConfigReset::new(default_config.clone());
    event_publisher.publish(Box::new(event));

    Ok(default_config)
}

pub async fn reset_config_to_factory_defaults(
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<Config> {
    let factory_config = Config::default();

    config_repo.save_config(&factory_config).await?;

    // Publish ConfigReset event for factory defaults
    let event = ConfigReset::new(factory_config.clone());
    event_publisher.publish(Box::new(event));

    Ok(factory_config)
}

pub async fn backup_and_reset_config(
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<(Config, Config)> {
    let backup_config = if config_repo.config_exists().await? {
        config_repo.get_config().await?
    } else {
        Config::default()
    };

    let new_config = reset_config(config_repo, event_publisher).await?;

    Ok((backup_config, new_config))
}
