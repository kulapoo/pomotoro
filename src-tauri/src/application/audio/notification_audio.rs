use pomotoro_domain::{AudioCategory, PlaybackHandle, Result, Error};
use super::play_audio::{PlayAudioCmd, play_audio};
use pomotoro_domain::AudioService;
use super::manage_library::{AudioLibraryService, get_assets_by_category};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct PlayNotificationSoundCmd {
    pub notification_type: NotificationType,
    pub volume: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct PlayBackgroundAudioCmd {
    pub category: AudioCategory,
    pub volume: Option<f32>,
    pub asset_id: Option<String>, // If None, picks default for category
}

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationType {
    SessionCompleted,
    BreakCompleted,
    TaskCompleted,
    PhaseTransition,
    Warning,
    Success,
}

impl NotificationType {
    pub fn to_asset_category(&self) -> AudioCategory {
        match self {
            NotificationType::SessionCompleted => AudioCategory::NotificationSound,
            NotificationType::BreakCompleted => AudioCategory::NotificationSound,
            NotificationType::TaskCompleted => AudioCategory::NotificationSound,
            NotificationType::PhaseTransition => AudioCategory::NotificationSound,
            NotificationType::Warning => AudioCategory::NotificationSound,
            NotificationType::Success => AudioCategory::NotificationSound,
        }
    }

    pub fn default_asset_id(&self) -> &'static str {
        match self {
            NotificationType::SessionCompleted => "session-complete-bell",
            NotificationType::BreakCompleted => "break-complete-chime",
            NotificationType::TaskCompleted => "task-complete-success",
            NotificationType::PhaseTransition => "phase-transition-soft",
            NotificationType::Warning => "warning-tone",
            NotificationType::Success => "success-chime",
        }
    }
}

pub async fn play_notification_sound(
    audio_service: &Arc<Mutex<dyn AudioService>>,
    library_service: &Arc<Mutex<dyn AudioLibraryService>>,
    cmd: PlayNotificationSoundCmd,
) -> Result<PlaybackHandle> {
    let volume = cmd.volume.unwrap_or(0.7);
    
    // Validate volume
    if !(0.0..=1.0).contains(&volume) {
        return Err(Error::ConfigurationError { 
            message: format!("Volume must be between 0.0 and 1.0, got {}", volume)
        });
    }

    // Get notification assets for this type
    let category = cmd.notification_type.to_asset_category();
    let assets = get_assets_by_category(library_service, category).await?;
    
    // Find the specific asset or use default
    let asset_id = if assets.is_empty() {
        // No assets found, use default asset ID (might be built-in)
        cmd.notification_type.default_asset_id().to_string()
    } else {
        // Try to find specific asset for notification type
        assets.iter()
            .find(|asset| asset.id == cmd.notification_type.default_asset_id())
            .map(|asset| asset.id.clone())
            .unwrap_or_else(|| assets[0].id.clone()) // Use first available
    };

    let play_cmd = PlayAudioCmd {
        asset_id,
        volume,
        looped: false, // Notifications should not loop
        fade_in_ms: Some(100), // Quick fade in for smoother sound
    };

    play_audio(audio_service, play_cmd).await
}

pub async fn play_background_audio(
    audio_service: &Arc<Mutex<dyn AudioService>>,
    library_service: &Arc<Mutex<dyn AudioLibraryService>>,
    cmd: PlayBackgroundAudioCmd,
) -> Result<PlaybackHandle> {
    let volume = cmd.volume.unwrap_or(0.3); // Lower default volume for background
    
    // Validate volume
    if !(0.0..=1.0).contains(&volume) {
        return Err(Error::ConfigurationError { 
            message: format!("Volume must be between 0.0 and 1.0, got {}", volume)
        });
    }

    // Get assets for the specified category
    let assets = get_assets_by_category(library_service, cmd.category).await?;
    
    if assets.is_empty() {
        return Err(Error::ConfigurationError {
            message: format!("No audio assets found for category: {:?}", cmd.category),
        });
    }

    // Select asset
    let asset_id = if let Some(specific_id) = cmd.asset_id {
        // Use specific asset if provided
        if assets.iter().any(|asset| asset.id == specific_id) {
            specific_id
        } else {
            return Err(Error::ConfigurationError {
                message: format!("Asset '{}' not found in category '{:?}'", specific_id, cmd.category),
            });
        }
    } else {
        // Use first available asset for category
        assets[0].id.clone()
    };

    let play_cmd = PlayAudioCmd {
        asset_id,
        volume,
        looped: true, // Background audio should loop
        fade_in_ms: Some(1000), // Longer fade in for background audio
    };

    play_audio(audio_service, play_cmd).await
}

pub async fn stop_background_audio(
    audio_service: &Arc<Mutex<dyn AudioService>>,
    _category: AudioCategory,
) -> Result<()> {
    let service = audio_service.lock().map_err(|e| Error::ConfigurationError {
        message: format!("Failed to acquire audio service lock: {}", e),
    })?;

    // Get all active playbacks
    let active_playbacks = service.get_active_playbacks()?;
    
    // Stop all looped playbacks (background audio)
    drop(service); // Release lock before calling stop_audio
    
    for playback in active_playbacks {
        if playback.is_looped {
            let stop_cmd = super::play_audio::StopAudioCmd {
                playback_id: playback.id,
            };
            super::play_audio::stop_audio(audio_service, stop_cmd).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pomotoro_domain::AudioService;
    use super::super::manage_library::AudioLibraryService;
    use pomotoro_domain::{AudioLibrary, AudioAsset, PlaybackHandle, PlaybackRequest};
    use std::collections::HashMap;

    struct MockAudioService {
        playbacks: HashMap<String, PlaybackHandle>,
    }

    impl MockAudioService {
        fn new() -> Self {
            Self {
                playbacks: HashMap::new(),
            }
        }
    }

    impl AudioService for MockAudioService {
        fn play_audio(&mut self, request: PlaybackRequest) -> Result<PlaybackHandle> {
            let handle = PlaybackHandle {
                id: format!("test-{}", uuid::Uuid::new_v4()),
                asset_id: request.asset_id,
                is_playing: true,
                is_looped: request.looped,
                volume: request.volume,
            };
            self.playbacks.insert(handle.id.clone(), handle.clone());
            Ok(handle)
        }

        fn stop_audio(&mut self, playback_id: &str) -> Result<()> {
            if let Some(mut handle) = self.playbacks.get_mut(playback_id) {
                handle.is_playing = false;
            }
            Ok(())
        }

        fn stop_all_audio(&mut self) -> Result<()> {
            for handle in self.playbacks.values_mut() {
                handle.is_playing = false;
            }
            Ok(())
        }

        fn pause_audio(&mut self, playback_id: &str) -> Result<()> {
            if let Some(mut handle) = self.playbacks.get_mut(playback_id) {
                handle.is_playing = false;
            }
            Ok(())
        }

        fn resume_audio(&mut self, playback_id: &str) -> Result<()> {
            if let Some(mut handle) = self.playbacks.get_mut(playback_id) {
                handle.is_playing = true;
            }
            Ok(())
        }

        fn set_volume(&mut self, playback_id: &str, volume: f32) -> Result<()> {
            if let Some(mut handle) = self.playbacks.get_mut(playback_id) {
                handle.volume = volume;
            }
            Ok(())
        }

        fn get_active_playbacks(&self) -> Result<Vec<PlaybackHandle>> {
            Ok(self.playbacks.values()
                .filter(|h| h.is_playing)
                .cloned()
                .collect())
        }

        fn cleanup_finished(&mut self) -> Result<()> {
            self.playbacks.retain(|_, handle| handle.is_playing);
            Ok(())
        }
    }

    struct MockLibraryService {
        library: AudioLibrary,
    }

    impl MockLibraryService {
        fn new_with_notification_assets() -> Self {
            let mut library = AudioLibrary::new();
            
            let asset = AudioAsset {
                id: "session-complete-bell".to_string(),
                name: "Session Complete Bell".to_string(),
                file_path: "/fake/path/bell.mp3".into(),
                category: AudioCategory::NotificationSound,
                duration_ms: Some(2000),
            };
            library.add_asset(asset);
            
            Self { library }
        }
    }

    impl AudioLibraryService for MockLibraryService {
        fn get_library(&self) -> Result<AudioLibrary> {
            Ok(self.library.clone())
        }

        fn add_asset(&mut self, asset: AudioAsset) -> Result<()> {
            self.library.add_asset(asset);
            Ok(())
        }

        fn remove_asset(&mut self, asset_id: &str) -> Result<bool> {
            Ok(self.library.remove_asset(asset_id).is_some())
        }

        fn get_asset(&self, asset_id: &str) -> Result<Option<AudioAsset>> {
            Ok(self.library.get_asset(asset_id).cloned())
        }

        fn get_assets_by_category(&self, category: AudioCategory) -> Result<Vec<AudioAsset>> {
            Ok(self.library.assets
                .values()
                .filter(|asset| asset.category == category)
                .cloned()
                .collect())
        }

        fn save_library(&self, _library: &AudioLibrary) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn should_play_notification_sound() {
        let audio_service: Arc<Mutex<dyn AudioService>> = Arc::new(Mutex::new(MockAudioService::new()));
        let library_service: Arc<Mutex<dyn AudioLibraryService>> = Arc::new(Mutex::new(MockLibraryService::new_with_notification_assets()));
        
        let cmd = PlayNotificationSoundCmd {
            notification_type: NotificationType::SessionCompleted,
            volume: Some(0.8),
        };

        let handle = play_notification_sound(&audio_service, &library_service, cmd).await.unwrap();
        
        assert_eq!(handle.asset_id, "session-complete-bell");
        assert_eq!(handle.volume, 0.8);
        assert!(!handle.is_looped); // Notifications should not loop
        assert!(handle.is_playing);
    }

    #[tokio::test]
    async fn should_fail_with_invalid_volume() {
        let audio_service: Arc<Mutex<dyn AudioService>> = Arc::new(Mutex::new(MockAudioService::new()));
        let library_service: Arc<Mutex<dyn AudioLibraryService>> = Arc::new(Mutex::new(MockLibraryService::new_with_notification_assets()));
        
        let cmd = PlayNotificationSoundCmd {
            notification_type: NotificationType::SessionCompleted,
            volume: Some(1.5), // Invalid volume
        };

        let result = play_notification_sound(&audio_service, &library_service, cmd).await;
        assert!(result.is_err());
    }
}