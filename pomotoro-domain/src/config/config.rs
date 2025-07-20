use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::{TaskConfig, AudioConfig, Error, Result, General, Notification, Appearance};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub task_config: TaskConfig,
    pub audio_config: AudioConfig,
    pub general: General,
    pub notification: Notification,
    pub appearance: Appearance,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            task_config: TaskConfig::default(),
            audio_config: AudioConfig::default(),
            general: General::default(),
            notification: Notification::default(),
            appearance: Appearance::default(),
        }
    }
}

impl Config {
    pub fn validate(&self) -> Result<()> {
        self.task_config.validate()?;
        self.audio_config.validate()?;

        if self.general.max_sessions_default == 0 || self.general.max_sessions_default > 10 {
            return Err(Error::InvalidSessionCount {
                count: self.general.max_sessions_default,
            });
        }

        if self.notification.auto_dismiss_delay_seconds > 300 {
            return Err(Error::InvalidDuration {
                duration: self.notification.auto_dismiss_delay_seconds,
            });
        }

        Ok(())
    }

    pub fn update_default_timings(&mut self, work_minutes: u32, short_break_minutes: u32, long_break_minutes: u32) {
        self.task_config.work_duration = Duration::from_secs((work_minutes * 60) as u64);
        self.task_config.short_break_duration = Duration::from_secs((short_break_minutes * 60) as u64);
        self.task_config.long_break_duration = Duration::from_secs((long_break_minutes * 60) as u64);
    }

    pub fn update_default_cycle_length(&mut self, sessions_until_long_break: u8) {
        self.task_config.sessions_until_long_break = sessions_until_long_break;
    }
}