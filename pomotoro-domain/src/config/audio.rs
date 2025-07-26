use serde::{Deserialize, Serialize};

use crate::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl AudioConfig {
    pub fn validate(&self) -> Result<()> {
        if self.volume < 0.0 || self.volume > 1.0 {
            return Err(Error::InvalidVolume { volume: self.volume });
        }
        Ok(())
    }
}