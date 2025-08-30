use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AudioCategory {
    NotificationSound,
    BackgroundAmbient,
    CustomUpload,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_category_equality() {
        assert_eq!(AudioCategory::NotificationSound, AudioCategory::NotificationSound);
        assert_ne!(AudioCategory::NotificationSound, AudioCategory::BackgroundAmbient);
        assert_ne!(AudioCategory::BackgroundAmbient, AudioCategory::CustomUpload);
    }

    #[test]
    fn test_audio_category_clone() {
        let original = AudioCategory::BackgroundAmbient;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_audio_category_copy() {
        let original = AudioCategory::CustomUpload;
        let copied = original;
        assert_eq!(original, copied);
    }

    #[test]
    fn test_audio_category_serialization() {
        let category = AudioCategory::NotificationSound;
        let serialized = serde_json::to_string(&category).unwrap();
        let deserialized: AudioCategory = serde_json::from_str(&serialized).unwrap();
        assert_eq!(category, deserialized);
    }

    #[test]
    fn test_all_audio_categories_serialization() {
        let categories = vec![
            AudioCategory::NotificationSound,
            AudioCategory::BackgroundAmbient,
            AudioCategory::CustomUpload,
        ];

        for category in categories {
            let serialized = serde_json::to_string(&category).unwrap();
            let deserialized: AudioCategory = serde_json::from_str(&serialized).unwrap();
            assert_eq!(category, deserialized);
        }
    }
}
