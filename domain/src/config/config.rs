use crate::{
    AppearanceConfig, AudioConfig, Error, GeneralConfig, NotificationConfig,
    Result, TaskDefaults,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub task_defaults: TaskDefaults,
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

    pub fn update_default_timings(
        &mut self,
        work_minutes: u32,
        short_break_minutes: u32,
        long_break_minutes: u32,
    ) -> Result<()> {
        self.task_defaults.update_timings(
            work_minutes,
            short_break_minutes,
            long_break_minutes,
        )
    }

    pub fn update_default_cycle_length(
        &mut self,
        sessions_until_long_break: u8,
    ) -> Result<()> {
        self.task_defaults
            .update_cycle_length(sessions_until_long_break)
    }
}
