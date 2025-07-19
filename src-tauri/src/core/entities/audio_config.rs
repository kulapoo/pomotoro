use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub work_notification_sound: Option<String>,
    pub break_notification_sound: Option<String>,
    pub background_sound: Option<String>,
    pub volume: f32,
    pub enable_background_audio: bool,
    pub muted: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            work_notification_sound: None,
            break_notification_sound: None,
            background_sound: None,
            volume: 0.7,
            enable_background_audio: false,
            muted: false,
        }
    }
}