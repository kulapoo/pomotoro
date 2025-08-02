use domain::TaskConfig;
use std::time::Duration;

pub struct ConfigFixtures;

impl ConfigFixtures {
    pub fn default_task_config() -> TaskConfig {
        TaskConfig::default()
    }

    pub fn custom_task_config() -> TaskConfig {
        TaskConfig {
            work_duration: Duration::from_secs(30 * 60), // 30 minutes
            short_break_duration: Duration::from_secs(7 * 60), // 7 minutes
            long_break_duration: Duration::from_secs(20 * 60), // 20 minutes
            sessions_until_long_break: 3,
            enable_screen_blocking: false,
        }
    }

    pub fn fast_task_config() -> TaskConfig {
        TaskConfig {
            work_duration: Duration::from_secs(5), // 5 seconds
            short_break_duration: Duration::from_secs(2), // 2 seconds
            long_break_duration: Duration::from_secs(3), // 3 seconds
            sessions_until_long_break: 2,
            enable_screen_blocking: false,
        }
    }
}