use pomotoro_domain::AudioConfig;

pub struct AudioFixtures;

impl AudioFixtures {
    pub fn default_audio_config() -> AudioConfig {
        AudioConfig::default()
    }

    pub fn custom_audio_config() -> AudioConfig {
        AudioConfig {
            work_notification_sound: Some("custom-work-sound".to_string()),
            break_notification_sound: Some("custom-break-sound".to_string()),
            background_sound: Some("custom-background".to_string()),
            volume: 0.6,
            enable_background_audio: true,
            muted: false,
        }
    }

    pub fn silent_audio_config() -> AudioConfig {
        AudioConfig {
            work_notification_sound: None,
            break_notification_sound: None,
            background_sound: None,
            volume: 0.0,
            enable_background_audio: false,
            muted: true,
        }
    }
}