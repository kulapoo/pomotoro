use crate::{
    AppearanceConfig, AudioConfig, Error, GeneralConfig, NotificationConfig,
    Result, TimerConfiguration,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Config {
    pub timer: TimerConfiguration,
    pub audio: AudioConfig,
    pub general: GeneralConfig,
    pub notification: NotificationConfig,
    pub appearance: AppearanceConfig,
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        self.timer.validate()?;
        self.audio.validate()?;
        self.general.validate()?;
        self.appearance.validate()?;

        if self.notification.auto_dismiss_delay_seconds > 300 {
            return Err(Error::InvalidDuration {
                duration: self.notification.auto_dismiss_delay_seconds,
            });
        }

        Ok(())
    }
}
