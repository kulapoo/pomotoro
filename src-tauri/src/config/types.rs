use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::task::types::{TaskConfig, AudioConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub default_task_config: TaskConfig,
    pub default_audio_config: AudioConfig,
    pub app_preferences: AppPreferences,
    pub notification_preferences: NotificationPreferences,
    pub ui_preferences: UiPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPreferences {
    pub task_cycling_behavior: TaskCyclingBehavior,
    pub max_sessions_default: u8,
    pub auto_start_breaks: bool,
    pub auto_start_work_after_break: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub enable_desktop_notifications: bool,
    pub enable_sound_notifications: bool,
    pub show_phase_transition_notifications: bool,
    pub show_task_completion_notifications: bool,
    pub notification_position: NotificationPosition,
    pub auto_dismiss_delay_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    pub theme: Theme,
    pub show_seconds_in_display: bool,
    pub always_on_top: bool,
    pub compact_mode: bool,
    pub show_task_list_sidebar: bool,
    pub animate_progress: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskCyclingBehavior {
    Manual,
    AutoAdvance,
    RoundRobin,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationPosition {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
    Center,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for GlobalConfig {
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

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            task_cycling_behavior: TaskCyclingBehavior::Manual,
            max_sessions_default: 4,
            auto_start_breaks: true,
            auto_start_work_after_break: false,
            minimize_to_tray: true,
            start_minimized: false,
        }
    }
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            enable_desktop_notifications: true,
            enable_sound_notifications: true,
            show_phase_transition_notifications: true,
            show_task_completion_notifications: true,
            notification_position: NotificationPosition::TopRight,
            auto_dismiss_delay_seconds: 5,
        }
    }
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            show_seconds_in_display: true,
            always_on_top: false,
            compact_mode: false,
            show_task_list_sidebar: true,
            animate_progress: true,
        }
    }
}

impl GlobalConfig {
    pub fn update_default_timings(&mut self, work_minutes: u32, short_break_minutes: u32, long_break_minutes: u32) {
        self.default_task_config.work_duration = Duration::from_secs((work_minutes * 60) as u64);
        self.default_task_config.short_break_duration = Duration::from_secs((short_break_minutes * 60) as u64);
        self.default_task_config.long_break_duration = Duration::from_secs((long_break_minutes * 60) as u64);
    }

    pub fn update_default_cycle_length(&mut self, sessions_until_long_break: u8) {
        self.default_task_config.sessions_until_long_break = sessions_until_long_break;
    }
}