use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::category::AudioCategory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioAsset {
    pub id: String,
    pub name: String,
    pub file_path: PathBuf,
    pub category: AudioCategory,
    pub duration_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_asset_creation() {
        let asset = AudioAsset {
            id: "test-id".to_string(),
            name: "Test Sound".to_string(),
            file_path: PathBuf::from("/path/to/sound.mp3"),
            category: AudioCategory::NotificationSound,
            duration_ms: Some(5000),
        };

        assert_eq!(asset.id, "test-id");
        assert_eq!(asset.name, "Test Sound");
        assert_eq!(asset.file_path, PathBuf::from("/path/to/sound.mp3"));
        assert_eq!(asset.category, AudioCategory::NotificationSound);
        assert_eq!(asset.duration_ms, Some(5000));
    }

    #[test]
    fn test_audio_asset_clone() {
        let original = AudioAsset {
            id: "original-id".to_string(),
            name: "Original Sound".to_string(),
            file_path: PathBuf::from("/original/path.mp3"),
            category: AudioCategory::BackgroundAmbient,
            duration_ms: None,
        };

        let cloned = original.clone();

        assert_eq!(cloned.id, original.id);
        assert_eq!(cloned.name, original.name);
        assert_eq!(cloned.file_path, original.file_path);
        assert_eq!(cloned.category, original.category);
        assert_eq!(cloned.duration_ms, original.duration_ms);
    }

    #[test]
    fn test_audio_asset_serialization() {
        let asset = AudioAsset {
            id: "serialize-test".to_string(),
            name: "Serialize Sound".to_string(),
            file_path: PathBuf::from("/serialize/sound.mp3"),
            category: AudioCategory::CustomUpload,
            duration_ms: Some(3000),
        };

        let serialized = serde_json::to_string(&asset).unwrap();
        let deserialized: AudioAsset = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.id, asset.id);
        assert_eq!(deserialized.name, asset.name);
        assert_eq!(deserialized.file_path, asset.file_path);
        assert_eq!(deserialized.category, asset.category);
        assert_eq!(deserialized.duration_ms, asset.duration_ms);
    }
}
