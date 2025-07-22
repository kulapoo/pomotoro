use pomotoro_lib::task::models::Task;
use pomotoro_domain::{TaskConfig, AudioConfig};
use std::time::Duration;
use uuid::Uuid;

pub struct TaskFixtures;

impl TaskFixtures {
    pub fn default_task() -> Task {
        Task::new_default()
    }

    pub fn work_task() -> Task {
        Task::new("Work Project".to_string(), 4).unwrap()
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

        Task::new("Study Session".to_string(), 3).unwrap()
            .with_tags(vec!["study".to_string(), "learning".to_string()])
            .with_config(config).unwrap()
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

        Task::new("Creative Work".to_string(), 2).unwrap()
            .with_tags(vec!["creative".to_string(), "art".to_string()])
            .with_config(config).unwrap()
            .with_audio_config(audio_config).unwrap()
    }

    pub fn completed_task() -> Task {
        let mut task = Task::new("Completed Task".to_string(), 2).unwrap();
        task.increment_session().unwrap();
        task.increment_session().unwrap();
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

        Task::new("Exercise".to_string(), 1).unwrap()
            .with_tags(vec!["health".to_string(), "fitness".to_string()])
            .with_config(config).unwrap()
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