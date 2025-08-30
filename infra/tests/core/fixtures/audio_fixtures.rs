use domain::audio::{AudioAsset, AudioCategory, AudioLibrary, PlaybackRequest, PlaybackHandle};
use std::path::PathBuf;

/// Audio-related fixtures for testing
pub struct AudioFixtures;

impl AudioFixtures {
    /// Create a test audio asset
    pub fn asset(name: impl Into<String>) -> AudioAsset {
        let name = name.into();
        AudioAsset {
            id: format!("{}_id", &name),
            name: name.clone(),
            file_path: PathBuf::from(format!("/test/audio/{}.mp3", name)),
            category: AudioCategory::NotificationSound,
            duration_ms: Some(1000),
        }
    }

    /// Create a custom audio asset
    pub fn custom_asset(name: impl Into<String>, path: PathBuf) -> AudioAsset {
        let name = name.into();
        AudioAsset {
            id: format!("custom_{}", &name),
            name,
            file_path: path,
            category: AudioCategory::CustomUpload,
            duration_ms: Some(2000),
        }
    }

    /// Create a notification sound asset
    pub fn notification_sound(name: impl Into<String>) -> AudioAsset {
        let name = name.into();
        AudioAsset {
            id: format!("notif_{}", &name),
            name: name.clone(),
            file_path: PathBuf::from(format!("/sounds/notifications/{}.mp3", name)),
            category: AudioCategory::NotificationSound,
            duration_ms: Some(500),
        }
    }

    /// Create a background sound asset
    pub fn background_sound(name: impl Into<String>) -> AudioAsset {
        let name = name.into();
        AudioAsset {
            id: format!("bg_{}", &name),
            name: name.clone(),
            file_path: PathBuf::from(format!("/sounds/background/{}.mp3", name)),
            category: AudioCategory::BackgroundAmbient,
            duration_ms: None,
        }
    }

    /// Create a test audio library with default sounds
    pub fn library() -> AudioLibrary {
        let mut library = AudioLibrary::new();
        
        // Add notification sounds
        library.add_asset(Self::notification_sound("bell"));
        library.add_asset(Self::notification_sound("chime"));
        library.add_asset(Self::notification_sound("ding"));
        
        // Add background sounds
        library.add_asset(Self::background_sound("rain"));
        library.add_asset(Self::background_sound("ocean"));
        library.add_asset(Self::background_sound("forest"));
        
        library
    }

    /// Create a playback request
    pub fn playback_request(asset_id: impl Into<String>, volume: f32) -> PlaybackRequest {
        PlaybackRequest {
            asset_id: asset_id.into(),
            volume,
            looped: false,
            fade_in_ms: None,
            fade_out_ms: None,
        }
    }

    /// Create a looped playback request (for background audio)
    pub fn looped_playback_request(asset_id: impl Into<String>, volume: f32) -> PlaybackRequest {
        PlaybackRequest {
            asset_id: asset_id.into(),
            volume,
            looped: true,
            fade_in_ms: Some(500),
            fade_out_ms: Some(500),
        }
    }

    /// Create a test playback handle
    pub fn playback_handle(id: impl Into<String>) -> PlaybackHandle {
        PlaybackHandle {
            id: id.into(),
            asset_id: "test_asset".to_string(),
            is_playing: true,
            is_looped: false,
            volume: 0.8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_audio_asset() {
        let asset = AudioFixtures::asset("test_sound");
        assert_eq!(asset.name, "test_sound");
        assert_eq!(asset.id, "test_sound_id");
        assert_eq!(asset.category, AudioCategory::NotificationSound);
    }

    #[test]
    fn creates_custom_asset() {
        let path = PathBuf::from("/custom/path/sound.mp3");
        let asset = AudioFixtures::custom_asset("custom", path.clone());
        assert_eq!(asset.file_path, path);
        assert_eq!(asset.category, AudioCategory::CustomUpload);
    }

    #[test]
    fn creates_audio_library() {
        let library = AudioFixtures::library();
        assert_eq!(library.assets.len(), 6);
        
        let notifications: usize = library.assets.values()
            .filter(|a| a.category == AudioCategory::NotificationSound)
            .count();
        assert_eq!(notifications, 3);
        
        let background: usize = library.assets.values()
            .filter(|a| a.category == AudioCategory::BackgroundAmbient)
            .count();
        assert_eq!(background, 3);
    }

    #[test]
    fn creates_playback_requests() {
        let normal = AudioFixtures::playback_request("sound1", 0.8);
        assert!(!normal.looped);
        assert_eq!(normal.volume, 0.8);

        let looped = AudioFixtures::looped_playback_request("background", 0.3);
        assert!(looped.looped);
        assert_eq!(looped.volume, 0.3);
    }
}