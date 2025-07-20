use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NotificationPosition {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
    Center,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub enable_desktop_notifications: bool,
    pub enable_sound_notifications: bool,
    pub show_phase_transition_notifications: bool,
    pub show_task_completion_notifications: bool,
    pub notification_position: NotificationPosition,
    pub auto_dismiss_delay_seconds: u32,
}

impl Default for Notification {
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