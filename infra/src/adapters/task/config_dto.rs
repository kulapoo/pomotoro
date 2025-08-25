use domain::{Result, TaskConfig, TaskDefaults};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfigDto {
    #[serde(with = "duration_serde")]
    pub work_duration: Duration,
    #[serde(with = "duration_serde")]
    pub short_break_duration: Duration,
    #[serde(with = "duration_serde")]
    pub long_break_duration: Duration,
    pub sessions_until_long_break: u8,
    pub enable_screen_blocking: bool,
    pub max_sessions_default: u8,
}

impl From<TaskDefaults> for TaskConfigDto {
    fn from(defaults: TaskDefaults) -> Self {
        Self {
            work_duration: defaults.work_duration,
            short_break_duration: defaults.short_break_duration,
            long_break_duration: defaults.long_break_duration,
            sessions_until_long_break: defaults.sessions_until_long_break,
            enable_screen_blocking: defaults.enable_screen_blocking,
            max_sessions_default: defaults.max_sessions_default,
        }
    }
}

impl From<TaskConfig> for TaskConfigDto {
    fn from(config: TaskConfig) -> Self {
        Self {
            work_duration: config.work_duration(),
            short_break_duration: config.short_break_duration(),
            long_break_duration: config.long_break_duration(),
            sessions_until_long_break: config.sessions_until_long_break(),
            enable_screen_blocking: config.enable_screen_blocking(),
            max_sessions_default: 1, // Default value for TaskConfig conversion since TaskConfig doesn't have this field
        }
    }
}

impl TryFrom<TaskConfigDto> for TaskDefaults {
    type Error = domain::Error;

    fn try_from(dto: TaskConfigDto) -> Result<Self> {
        TaskDefaults::new(
            dto.work_duration,
            dto.short_break_duration,
            dto.long_break_duration,
            dto.sessions_until_long_break,
            dto.enable_screen_blocking,
            dto.max_sessions_default,
        )
    }
}

impl TryFrom<TaskConfigDto> for TaskConfig {
    type Error = domain::Error;

    fn try_from(dto: TaskConfigDto) -> Result<Self> {
        TaskConfig::new(
            dto.work_duration,
            dto.short_break_duration,
            dto.long_break_duration,
            dto.sessions_until_long_break,
            dto.enable_screen_blocking,
        )
    }
}

mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(
        duration: &Duration,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}
