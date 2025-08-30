use crate::{
    AppearanceConfig, AudioConfig, Error, GeneralConfig, NotificationConfig,
    Result,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub audio: AudioConfig,
    pub general: GeneralConfig,
    pub notification: NotificationConfig,
    pub appearance: AppearanceConfig,
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        self.audio.validate()?;

        if self.notification.auto_dismiss_delay_seconds > 300 {
            return Err(Error::InvalidDuration {
                duration: self.notification.auto_dismiss_delay_seconds,
            });
        }

        Ok(())
    }

}
