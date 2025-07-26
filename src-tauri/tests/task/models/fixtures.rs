use pomotoro_domain::{Task, TaskBuilder, TaskConfig, AudioConfig, TaskDefaults};
use std::time::Duration;
use pomotoro_domain::TaskId;

pub struct TaskFixtures;

impl TaskFixtures {
    fn defaults() -> TaskDefaults {
        TaskDefaults::default()
    }

    pub fn default_task() -> Task {
        Task::new_default(&Self::defaults()).unwrap()
    }

    pub fn work_task() -> Task {
        TaskBuilder::with_name_and_sessions("Work Project".to_string(), 4)
            .with_tags(vec!["work".to_string(), "project".to_string()])
            .with_description("Important work project".to_string())
            .build(&Self::defaults()).unwrap()
    }

    pub fn study_task() -> Task {
        let config = TaskConfig {
            work_duration: Duration::from_secs(50 * 60), // 50 minutes
            short_break_duration: Duration::from_secs(10 * 60), // 10 minutes
            long_break_duration: Duration::from_secs(30 * 60), // 30 minutes
            sessions_until_long_break: 3,
            enable_screen_blocking: true,
        };

        TaskBuilder::with_name_and_sessions("Study Session".to_string(), 3)
            .with_tags(vec!["study".to_string(), "learning".to_string()])
            .with_config(config)
            .build(&Self::defaults()).unwrap()
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

        TaskBuilder::with_name_and_sessions("Creative Work".to_string(), 2)
            .with_tags(vec!["creative".to_string(), "art".to_string()])
            .with_config(config)
            .with_audio_config(audio_config)
            .build(&Self::defaults()).unwrap()
    }

    pub fn completed_task() -> Task {
        let mut task = Task::new("Completed Task".to_string(), 2, &Self::defaults()).unwrap();
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

        TaskBuilder::with_name_and_sessions("Exercise".to_string(), 1)
            .with_tags(vec!["health".to_string(), "fitness".to_string()])
            .with_config(config)
            .build(&Self::defaults()).unwrap()
    }
}

pub struct TestIds;

impl TestIds {
    pub fn random_task_id() -> TaskId {
        TaskId::new()
    }

    pub fn default_task_id() -> TaskId {
        // Use a fixed TaskId for the default task in tests
        TaskId::from_string("00000000-0000-0000-0000-000000000001").unwrap()
    }

    pub fn work_task_id() -> TaskId {
        TaskId::from_string("00000000-0000-0000-0000-000000000002").unwrap()
    }

    pub fn study_task_id() -> TaskId {
        TaskId::from_string("00000000-0000-0000-0000-000000000003").unwrap()
    }
}