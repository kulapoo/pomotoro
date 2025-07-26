use pomotoro_domain::{Config, ConfigRepository, EventPublisher, Result};
use std::sync::Arc;

pub async fn reset_config(
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
    _event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<Config> {
    // Reset to defaults using repository method
    let default_config = config_repo.reset_to_defaults().await?;
    
    // TODO: Publish ConfigReset event when domain events are implemented
    
    Ok(default_config)
}

pub async fn reset_config_to_factory_defaults(
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
    _event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<Config> {
    // Create fresh default config
    let factory_config = Config::default();
    
    // Save it to repository
    config_repo.save_config(&factory_config).await?;
    
    // TODO: Publish ConfigFactoryReset event when domain events are implemented
    
    Ok(factory_config)
}

pub async fn backup_and_reset_config(
    config_repo: &Arc<dyn ConfigRepository + Send + Sync>,
    event_publisher: &Arc<dyn EventPublisher + Send + Sync>,
) -> Result<(Config, Config)> {
    // Get current config as backup
    let backup_config = if config_repo.config_exists().await? {
        config_repo.get_config().await?
    } else {
        Config::default()
    };
    
    // Reset to defaults
    let new_config = reset_config(config_repo, event_publisher).await?;
    
    Ok((backup_config, new_config))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pomotoro_domain::NoOpEventPublisher;
    use crate::infrastructure::InMemoryConfigRepository;

    async fn setup() -> (Arc<dyn ConfigRepository + Send + Sync>, Arc<dyn EventPublisher + Send + Sync>) {
        let config_repo: Arc<dyn ConfigRepository + Send + Sync> = Arc::new(InMemoryConfigRepository::new());
        let event_publisher: Arc<dyn EventPublisher + Send + Sync> = Arc::new(NoOpEventPublisher);
        
        (config_repo, event_publisher)
    }

    #[tokio::test]
    async fn should_reset_config_to_defaults() {
        let (config_repo, event_publisher) = setup().await;
        
        // Create and save a modified config
        let mut custom_config = Config::default();
        custom_config.general.auto_start_breaks = false;
        custom_config.audio.volume = 0.3;
        config_repo.save_config(&custom_config).await.unwrap();
        
        // Verify custom config was saved
        let saved_config = config_repo.get_config().await.unwrap();
        assert!(!saved_config.general.auto_start_breaks);
        assert_eq!(saved_config.audio.volume, 0.3);
        
        // Reset to defaults
        let reset_config = reset_config(&config_repo, &event_publisher)
            .await
            .unwrap();
        
        // Should be back to default values
        assert!(reset_config.general.auto_start_breaks); // Default
        assert_eq!(reset_config.audio.volume, 0.7); // Default
    }

    #[tokio::test]
    async fn should_reset_to_factory_defaults() {
        let (config_repo, event_publisher) = setup().await;
        
        // Create modified config
        let mut custom_config = Config::default();
        custom_config.general.auto_start_breaks = false;
        config_repo.save_config(&custom_config).await.unwrap();
        
        // Reset to factory defaults
        let factory_config = reset_config_to_factory_defaults(&config_repo, &event_publisher)
            .await
            .unwrap();
        
        // Should match fresh default config
        let expected_config = Config::default();
        assert_eq!(factory_config.general.auto_start_breaks, 
                   expected_config.general.auto_start_breaks);
        assert_eq!(factory_config.audio.volume, 
                   expected_config.audio.volume);
        
        // Verify it was actually saved
        let saved_config = config_repo.get_config().await.unwrap();
        assert!(saved_config.general.auto_start_breaks);
    }

    #[tokio::test]
    async fn should_backup_and_reset_config() {
        let (config_repo, event_publisher) = setup().await;
        
        // Create custom config
        let mut custom_config = Config::default();
        custom_config.general.auto_start_breaks = false;
        custom_config.audio.volume = 0.7;
        config_repo.save_config(&custom_config).await.unwrap();
        
        // Backup and reset
        let (backup, new_config) = backup_and_reset_config(&config_repo, &event_publisher)
            .await
            .unwrap();
        
        // Backup should contain the old custom values
        assert!(!backup.general.auto_start_breaks);
        assert_eq!(backup.audio.volume, 0.7);
        
        // New config should be defaults
        assert!(new_config.general.auto_start_breaks);
        assert_eq!(new_config.audio.volume, 0.7);
    }

    #[tokio::test]
    async fn should_handle_reset_when_no_config_exists() {
        let (config_repo, event_publisher) = setup().await;
        
        // Don't create any initial config
        assert!(!config_repo.config_exists().await.unwrap());
        
        // Reset should still work
        let reset_config = reset_config(&config_repo, &event_publisher)
            .await
            .unwrap();
        
        // Should be default values
        assert!(reset_config.general.auto_start_breaks);
        assert_eq!(reset_config.audio.volume, 0.7);
        
        // Config should now exist
        assert!(config_repo.config_exists().await.unwrap());
    }

    #[tokio::test]
    async fn should_handle_backup_when_no_config_exists() {
        let (config_repo, event_publisher) = setup().await;
        
        // Don't create any initial config
        assert!(!config_repo.config_exists().await.unwrap());
        
        // Backup and reset should still work
        let (backup, new_config) = backup_and_reset_config(&config_repo, &event_publisher)
            .await
            .unwrap();
        
        // Backup should be default (since no config existed)
        assert!(backup.general.auto_start_breaks);
        
        // New config should also be default
        assert!(new_config.general.auto_start_breaks);
    }

    #[tokio::test]
    async fn should_preserve_all_config_sections_in_backup() {
        let (config_repo, event_publisher) = setup().await;
        
        // Create custom config with changes in multiple sections
        let mut custom_config = Config::default();
        custom_config.general.auto_start_breaks = false;
        custom_config.audio.volume = 0.4;
        custom_config.notification.enable_desktop_notifications = false;
        config_repo.save_config(&custom_config).await.unwrap();
        
        let (backup, _) = backup_and_reset_config(&config_repo, &event_publisher)
            .await
            .unwrap();
        
        // All custom values should be preserved in backup
        assert!(!backup.general.auto_start_breaks);
        assert_eq!(backup.audio.volume, 0.4);
        assert!(!backup.notification.enable_desktop_notifications);
    }
}