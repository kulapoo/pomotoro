use pomotoro_lib::task::models::{Task, TaskConfig, AudioConfig};
use pomotoro_lib::config::models::{GlobalConfig, AppPreferences, NotificationPreferences, UiPreferences, TaskCyclingBehavior};
use std::time::Duration;
use uuid::Uuid;

pub struct TaskFixtures;

impl TaskFixtures {
    pub fn default_task() -> Task {
        Task::new_default()
    }

    pub fn work_task() -> Task {
        Task::new("Work Project".to_string(), 4)
            .with_tags(vec!["work".to_string(), "project".to_string()])
            .with_description("Important work project".to_string())
    }

    pub fn study_task() -> Task {
        let config = TaskConfig {
            work_duration: Duration::from_secs(50 * 60), // 50 minutes
            short_break_duration: Duration::from_secs(10 * 60), // 10 minutes
            long_break_duration: Duration::from_secs(30 * 60), // 30 minutes
            sessions_until_long_break: 3,
            enable_screen_blocking: true,
        };

        Task::new("Study Session".to_string(), 3)
            .with_tags(vec!["study".to_string(), "learning".to_string()])
            .with_config(config)
    }

    pub fn creative_task() -> Task {
        let config = TaskConfig {
            work_duration: Duration::from_secs(15 * 60), // 15 minutes
            short_break_duration: Duration::from_secs(5 * 60), // 5 minutes
            long_break_duration: Duration::from_secs(15 * 60), // 15 minutes
            sessions_until_long_break: 2,
            enable_screen_blocking: false,
        };

        let audio_config = AudioConfig {
            work_notification_sound: Some("gentle-chime".to_string()),
            break_notification_sound: Some("soft-bell".to_string()),
            background_sound: Some("nature-sounds".to_string()),
            volume: 0.7,
            enable_background_audio: true,
            muted: false,
        };

        Task::new("Creative Work".to_string(), 2)
            .with_tags(vec!["creative".to_string(), "art".to_string()])
            .with_config(config)
            .with_audio_config(audio_config)
    }

    pub fn completed_task() -> Task {
        let mut task = Task::new("Completed Task".to_string(), 2);
        task.increment_session();
        task.increment_session();
        task
    }

    pub fn exercise_task() -> Task {
        let config = TaskConfig {
            work_duration: Duration::from_secs(30 * 60), // 30 minutes
            short_break_duration: Duration::from_secs(2 * 60), // 2 minutes
            long_break_duration: Duration::from_secs(10 * 60), // 10 minutes
            sessions_until_long_break: 1,
            enable_screen_blocking: false,
        };

        Task::new("Exercise".to_string(), 1)
            .with_tags(vec!["health".to_string(), "fitness".to_string()])
            .with_config(config)
    }
}

pub struct ConfigFixtures;

impl ConfigFixtures {
    pub fn default_global_config() -> GlobalConfig {
        GlobalConfig::default()
    }

    pub fn custom_global_config() -> GlobalConfig {
        let task_config = TaskConfig {
            work_duration: Duration::from_secs(30 * 60), // 30 minutes
            short_break_duration: Duration::from_secs(7 * 60), // 7 minutes
            long_break_duration: Duration::from_secs(20 * 60), // 20 minutes
            sessions_until_long_break: 3,
            enable_screen_blocking: false,
        };

        let audio_config = AudioConfig {
            work_notification_sound: Some("work-bell".to_string()),
            break_notification_sound: Some("break-chime".to_string()),
            background_sound: Some("focus-sounds".to_string()),
            volume: 0.8,
            enable_background_audio: true,
            muted: false,
        };

        let app_preferences = AppPreferences {
            task_cycling_behavior: TaskCyclingBehavior::AutoAdvance,
            auto_start_next_session: true,
            auto_start_breaks: false,
            minimize_on_start: false,
            start_minimized: false,
            close_to_tray: true,
            show_desktop_notifications: true,
            play_notification_sounds: true,
        };

        let notification_preferences = NotificationPreferences {
            show_work_start_notification: true,
            show_work_end_notification: true,
            show_break_start_notification: true,
            show_break_end_notification: true,
            show_task_completion_notification: true,
            notification_duration_seconds: 5,
            enable_sound: true,
            enable_system_integration: true,
        };

        let ui_preferences = UiPreferences {
            theme: "dark".to_string(),
            always_on_top: false,
            show_seconds: true,
            show_progress_ring: true,
            show_session_count: true,
            compact_mode: false,
            sidebar_collapsed: false,
            window_opacity: 1.0,
        };

        GlobalConfig {
            default_task_config: task_config,
            default_audio_config: audio_config,
            app_preferences,
            notification_preferences,
            ui_preferences,
        }
    }
}

pub struct AudioFixtures;

impl AudioFixtures {
    pub fn default_audio_config() -> AudioConfig {
        AudioConfig::default()
    }

    pub fn custom_audio_config() -> AudioConfig {
        AudioConfig {
            work_notification_sound: Some("custom-work-sound".to_string()),
            break_notification_sound: Some("custom-break-sound".to_string()),
            background_sound: Some("custom-background".to_string()),
            volume: 0.6,
            enable_background_audio: true,
            muted: false,
        }
    }

    pub fn silent_audio_config() -> AudioConfig {
        AudioConfig {
            work_notification_sound: None,
            break_notification_sound: None,
            background_sound: None,
            volume: 0.0,
            enable_background_audio: false,
            muted: true,
        }
    }
}

pub struct TestIds;

impl TestIds {
    pub fn random_task_id() -> Uuid {
        Uuid::new_v4()
    }

    pub fn default_task_id() -> Uuid {
        // Use a fixed UUID for the default task in tests
        Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
    }

    pub fn work_task_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
    }

    pub fn study_task_id() -> Uuid {
        Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap()
    }
}