use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::core::entities::{TaskConfig, AudioConfig};
use super::{AppPreferences, NotificationPreferences, UiPreferences};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_task_config: TaskConfig,
    pub default_audio_config: AudioConfig,
    pub app_preferences: AppPreferences,
    pub notification_preferences: NotificationPreferences,
    pub ui_preferences: UiPreferences,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_task_config: TaskConfig::default(),
            default_audio_config: AudioConfig::default(),
            app_preferences: AppPreferences::default(),
            notification_preferences: NotificationPreferences::default(),
            ui_preferences: UiPreferences::default(),
        }
    }
}

impl Config {
    pub fn update_default_timings(&mut self, work_minutes: u32, short_break_minutes: u32, long_break_minutes: u32) {
        self.default_task_config.work_duration = Duration::from_secs((work_minutes * 60) as u64);
        self.default_task_config.short_break_duration = Duration::from_secs((short_break_minutes * 60) as u64);
        self.default_task_config.long_break_duration = Duration::from_secs((long_break_minutes * 60) as u64);
    }

    pub fn update_default_cycle_length(&mut self, sessions_until_long_break: u8) {
        self.default_task_config.sessions_until_long_break = sessions_until_long_break;
    }
}
