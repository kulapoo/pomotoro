use domain::{
    AppearanceConfig, AudioConfig, Config, ConfigRepository, ConfigUpdated,
    EventPublisher, GeneralConfig, NotificationConfig, Result,
    TimerConfiguration,
};
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct UpdateConfigCmd {
    pub timer: Option<TimerConfiguration>,
    pub general: Option<GeneralConfig>,
    pub audio: Option<AudioConfig>,
    pub notification: Option<NotificationConfig>,
    pub appearance: Option<AppearanceConfig>,
}

pub async fn update_config(
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    cmd: UpdateConfigCmd,
) -> Result<Config> {
    let mut config = match config_repo.config_exists().await? {
        true => config_repo.get_config().await?,
        false => Config::default(),
    };

    if let Some(general) = cmd.general {
        general.validate()?;
        config.general = general;
    }

    if let Some(audio) = cmd.audio {
        audio.validate()?;
        config.audio = audio;
    }

    if let Some(notification) = cmd.notification {
        notification.validate()?;
        config.notification = notification;
    }

    if let Some(appearance) = cmd.appearance {
        appearance.validate()?;
        config.appearance = appearance;
    }

    config.validate()?;

    config_repo.save_config(&config).await?;

    event_publisher.publish(Box::new(ConfigUpdated::new(config.clone())));

    Ok(config)
}

pub async fn update_full_config(
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
    _event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    new_config: Config,
) -> Result<Config> {
    new_config.validate()?;

    config_repo.save_config(&new_config).await?;

    // TODO: Publish ConfigUpdated event when domain events are implemented

    Ok(new_config)
}

pub async fn update_general_config(
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    general_config: GeneralConfig,
) -> Result<Config> {
    let cmd = UpdateConfigCmd {
        timer: None,
        general: Some(general_config),
        audio: None,
        notification: None,
        appearance: None,
    };

    update_config(config_repo, event_publisher, cmd).await
}

pub async fn update_audio_config(
    config_repo: Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: Arc<dyn EventPublisher + Send + Sync>,
    audio_config: AudioConfig,
) -> Result<Config> {
    let cmd = UpdateConfigCmd {
        timer: None,
        general: None,
        audio: Some(audio_config),
        notification: None,
        appearance: None,
    };

    update_config(config_repo, event_publisher, cmd).await
}
