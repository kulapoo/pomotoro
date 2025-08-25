use domain::{
    AppearanceConfig, AudioConfig, Config, ConfigRepository, EventPublisher,
    GeneralConfig, NotificationConfig, Result,
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct UpdateConfigCmd {
    pub general: Option<GeneralConfig>,
    pub audio: Option<AudioConfig>,
    pub notification: Option<NotificationConfig>,
    pub appearance: Option<AppearanceConfig>,
}

pub async fn update_config(
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
    _event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
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

    // TODO: Publish ConfigUpdated event when domain events are implemented

    Ok(config)
}

pub async fn update_full_config(
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
    _event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    new_config: Config,
) -> Result<Config> {
    new_config.validate()?;

    config_repo.save_config(&new_config).await?;

    // TODO: Publish ConfigUpdated event when domain events are implemented

    Ok(new_config)
}

pub async fn update_general_config(
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    general_config: GeneralConfig,
) -> Result<Config> {
    let cmd = UpdateConfigCmd {
        general: Some(general_config),
        audio: None,
        notification: None,
        appearance: None,
    };

    update_config(config_repo, event_publisher, cmd).await
}

pub async fn update_audio_config(
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
    audio_config: AudioConfig,
) -> Result<Config> {
    let cmd = UpdateConfigCmd {
        general: None,
        audio: Some(audio_config),
        notification: None,
        appearance: None,
    };

    update_config(config_repo, event_publisher, cmd).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::InMemoryConfigRepository;
    use domain::NoOpEventPublisher;

    async fn setup() -> (
        Arc<dyn ConfigRepository + Send + Sync>,
        Arc<dyn EventPublisher + Send + Sync>,
    ) {
        let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
            Arc::new(InMemoryConfigRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> =
            Arc::new(NoOpEventPublisher);

        config_repo.save_config(&Config::default()).await.unwrap();

        (config_repo, event_publisher)
    }

    #[tokio::test]
    async fn should_update_general_config_only() {
        let (config_repo, event_publisher) = setup().await;

        let new_general = GeneralConfig {
            auto_start_breaks: false,
            ..Default::default()
        };

        let cmd = UpdateConfigCmd {
            general: Some(new_general),
            audio: None,
            notification: None,
            appearance: None,
        };

        let updated_config = update_config(&config_repo, &event_publisher, cmd)
            .await
            .unwrap();

        assert!(!updated_config.general.auto_start_breaks);
        assert_eq!(updated_config.audio, AudioConfig::default());
    }

    #[tokio::test]
    async fn should_update_multiple_config_sections() {
        let (config_repo, event_publisher) = setup().await;

        let new_general = GeneralConfig {
            minimize_to_tray: false,
            ..Default::default()
        };

        let new_audio = AudioConfig {
            volume: 0.8,
            ..Default::default()
        };

        let cmd = UpdateConfigCmd {
            general: Some(new_general),
            audio: Some(new_audio),
            notification: None,
            appearance: None,
        };

        let updated_config = update_config(&config_repo, &event_publisher, cmd)
            .await
            .unwrap();

        assert!(!updated_config.general.minimize_to_tray);
        assert_eq!(updated_config.audio.volume, 0.8);
        assert_eq!(
            updated_config.notification.enable_desktop_notifications,
            NotificationConfig::default().enable_desktop_notifications
        );
    }

    #[tokio::test]
    async fn should_update_full_config() {
        let (config_repo, event_publisher) = setup().await;

        let mut new_config = Config::default();
        new_config.general.start_minimized = true;
        new_config.audio.volume = 0.5;

        let updated_config = update_full_config(
            &config_repo,
            &event_publisher,
            new_config.clone(),
        )
        .await
        .unwrap();

        assert!(updated_config.general.start_minimized);
        assert_eq!(updated_config.audio.volume, 0.5);
    }

    #[tokio::test]
    async fn should_create_config_if_none_exists() {
        let config_repo: Arc<dyn ConfigRepository + Send + Sync> =
            Arc::new(InMemoryConfigRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> =
            Arc::new(NoOpEventPublisher);

        assert!(!config_repo.config_exists().await.unwrap());

        let new_general = GeneralConfig {
            start_minimized: true,
            ..Default::default()
        };

        let cmd = UpdateConfigCmd {
            general: Some(new_general),
            audio: None,
            notification: None,
            appearance: None,
        };

        let updated_config = update_config(&config_repo, &event_publisher, cmd)
            .await
            .unwrap();

        assert!(updated_config.general.start_minimized);
        assert!(config_repo.config_exists().await.unwrap());
    }

    #[tokio::test]
    async fn should_succeed_with_valid_general_config() {
        let (config_repo, event_publisher) = setup().await;

        let valid_general = GeneralConfig::default();

        let cmd = UpdateConfigCmd {
            general: Some(valid_general),
            audio: None,
            notification: None,
            appearance: None,
        };

        let result = update_config(&config_repo, &event_publisher, cmd).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_update_general_config_using_convenience_method() {
        let (config_repo, event_publisher) = setup().await;

        let new_general = GeneralConfig {
            auto_start_work_after_break: true,
            ..Default::default()
        };

        let updated_config =
            update_general_config(&config_repo, &event_publisher, new_general)
                .await
                .unwrap();

        assert!(updated_config.general.auto_start_work_after_break);
        assert_eq!(updated_config.audio, AudioConfig::default());
    }

    #[tokio::test]
    async fn should_update_audio_config_using_convenience_method() {
        let (config_repo, event_publisher) = setup().await;

        let new_audio = AudioConfig {
            volume: 0.3,
            ..Default::default()
        };

        let updated_config =
            update_audio_config(&config_repo, &event_publisher, new_audio)
                .await
                .unwrap();

        assert_eq!(updated_config.audio.volume, 0.3);
        assert_eq!(
            updated_config.general.auto_start_breaks,
            GeneralConfig::default().auto_start_breaks
        );
    }
}
