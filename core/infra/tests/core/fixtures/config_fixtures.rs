use domain::{
    AppearanceConfig, AudioConfig, Config, GeneralConfig, NotificationConfig,
    NotificationPosition, TaskCyclingBehavior, Theme, TimerConfiguration,
};

/// Configuration fixtures for testing
pub struct ConfigFixtures;

impl ConfigFixtures {
    /// Create a default config
    pub fn default() -> Config {
        Config::default()
    }

    /// Create a minimal config for testing
    pub fn minimal() -> Config {
        Config {
            timer: TimerConfiguration::default(),
            general: Self::minimal_general(),
            notification: Self::minimal_notification(),
            appearance: Self::minimal_appearance(),
            audio: Self::minimal_audio(),
        }
    }

    /// Create general config with test defaults
    pub fn minimal_general() -> GeneralConfig {
        GeneralConfig {
            task_cycling_behavior: TaskCyclingBehavior::Manual,
            auto_start_breaks: false,
            auto_start_work_after_break: false,
            minimize_to_tray: false,
            start_minimized: false,
            show_countdown_in_tray: true,
            persistence_interval_seconds: 10,
            block_screen_after_work: false,
            block_screen_after_work_message:
                "Work session complete. Time for a break.".to_string(),
            block_screen_after_break: false,
            block_screen_after_break_message: "Break over. Back to work."
                .to_string(),
        }
    }

    /// Create notification config for testing
    pub fn minimal_notification() -> NotificationConfig {
        NotificationConfig {
            enable_desktop_notifications: false,
            enable_sound_notifications: false,
            show_phase_transition_notifications: true,
            show_task_completion_notifications: true,
            notification_position: NotificationPosition::TopRight,
            auto_dismiss_delay_seconds: 5,
        }
    }

    /// Create appearance config for testing
    pub fn minimal_appearance() -> AppearanceConfig {
        AppearanceConfig {
            theme: Theme::Light,
        }
    }

    /// Create audio config for testing
    pub fn minimal_audio() -> AudioConfig {
        AudioConfig {
            work_notification_sound: None,
            break_notification_sound: None,
            background_sound: None,
            volume: 0.7,
            enable_background_audio: false,
            muted: false,
        }
    }

    /// Create task config for testing
    pub fn task_config() -> Config {
        Config::default()
    }

    /// Create a config optimized for fast testing
    pub fn fast_test_config() -> Config {
        Config::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_default_config() {
        let _config = ConfigFixtures::default();
        // Task defaults have been removed
    }

    #[test]
    fn creates_minimal_config() {
        let config = ConfigFixtures::minimal();
        assert!(!config.notification.enable_desktop_notifications);
        assert!(!config.general.auto_start_work_after_break);
    }
}
