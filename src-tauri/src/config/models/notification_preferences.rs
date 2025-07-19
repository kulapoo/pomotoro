use serde::{Deserialize, Serialize};
use crate::core::entities::NotificationPosition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub enable_desktop_notifications: bool,
    pub enable_sound_notifications: bool,
    pub show_phase_transition_notifications: bool,
    pub show_task_completion_notifications: bool,
    pub notification_position: NotificationPosition,
    pub auto_dismiss_delay_seconds: u32,
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