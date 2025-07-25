use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskConfig {
    #[serde(with = "duration_serde")]
    pub work_duration: Duration,
    #[serde(with = "duration_serde")]
    pub short_break_duration: Duration,
    #[serde(with = "duration_serde")]
    pub long_break_duration: Duration,
    pub sessions_until_long_break: u8,
    pub enable_screen_blocking: bool,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_until_long_break: 4,
            enable_screen_blocking: false,
        }
    }
}

impl TaskConfig {
    pub fn validate(&self) -> Result<()> {
        if self.work_duration.as_secs() < 60 || self.work_duration.as_secs() > 3600 {
            return Err(Error::InvalidDuration {
                duration: self.work_duration.as_secs() as u32,
            });
        }
        
        if self.short_break_duration.as_secs() < 30 || self.short_break_duration.as_secs() > 1800 {
            return Err(Error::InvalidDuration {
                duration: self.short_break_duration.as_secs() as u32,
            });
        }
        
        if self.long_break_duration.as_secs() < 300 || self.long_break_duration.as_secs() > 3600 {
            return Err(Error::InvalidDuration {
                duration: self.long_break_duration.as_secs() as u32,
            });
        }
        
        if self.sessions_until_long_break == 0 || self.sessions_until_long_break > 10 {
            return Err(Error::InvalidSessionCount {
                count: self.sessions_until_long_break,
            });
        }
        
        Ok(())
    }
}

// Conversion from TaskConfig to TimerConfiguration
impl Into<crate::TimerConfiguration> for TaskConfig {
    fn into(self) -> crate::TimerConfiguration {
        crate::TimerConfiguration {
            work_duration: self.work_duration,
            short_break_duration: self.short_break_duration,
            long_break_duration: self.long_break_duration,
            sessions_until_long_break: self.sessions_until_long_break,
        }
    }
}

impl From<crate::TimerConfiguration> for TaskConfig {
    fn from(timer_config: crate::TimerConfiguration) -> Self {
        Self {
            work_duration: timer_config.work_duration,
            short_break_duration: timer_config.short_break_duration,
            long_break_duration: timer_config.long_break_duration,
            sessions_until_long_break: timer_config.sessions_until_long_break,
            enable_screen_blocking: false, // Default value for new field
        }
    }
}

mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where 
        S: Serializer 
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where 
        D: Deserializer<'de> 
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}