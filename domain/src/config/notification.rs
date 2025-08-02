use serde::{Deserialize, Serialize};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationPosition {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
    Center,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub enable_desktop_notifications: bool,
    pub enable_sound_notifications: bool,
    pub show_phase_transition_notifications: bool,
    pub show_task_completion_notifications: bool,
    pub notification_position: NotificationPosition,
    pub auto_dismiss_delay_seconds: u32,
}

impl Default for NotificationConfig {
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

impl NotificationConfig {
    pub fn validate(&self) -> Result<()> {
        if self.auto_dismiss_delay_seconds > 300 {
            return Err(Error::InvalidDuration {
                duration: self.auto_dismiss_delay_seconds,
            });
        }
        Ok(())
    }
}