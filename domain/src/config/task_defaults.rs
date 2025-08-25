use crate::{Error, Result, duration_serde};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskDefaults {
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

impl Default for TaskDefaults {
    fn default() -> Self {
        Self {
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_until_long_break: 4,
            enable_screen_blocking: false,
            max_sessions_default: 4,
        }
    }
}

impl TaskDefaults {
    pub fn new(
        work_duration: Duration,
        short_break_duration: Duration,
        long_break_duration: Duration,
        sessions_until_long_break: u8,
        enable_screen_blocking: bool,
        max_sessions_default: u8,
    ) -> Result<Self> {
        Self::validate_durations(
            &work_duration,
            &short_break_duration,
            &long_break_duration,
        )?;
        Self::validate_session_count(sessions_until_long_break)?;
        Self::validate_max_sessions(max_sessions_default)?;

        Ok(Self {
            work_duration,
            short_break_duration,
            long_break_duration,
            sessions_until_long_break,
            enable_screen_blocking,
            max_sessions_default,
        })
    }

    pub fn update_timings(
        &mut self,
        work_minutes: u32,
        short_break_minutes: u32,
        long_break_minutes: u32,
    ) -> Result<()> {
        let work_duration = Duration::from_secs((work_minutes * 60) as u64);
        let short_break_duration =
            Duration::from_secs((short_break_minutes * 60) as u64);
        let long_break_duration =
            Duration::from_secs((long_break_minutes * 60) as u64);

        Self::validate_durations(
            &work_duration,
            &short_break_duration,
            &long_break_duration,
        )?;

        self.work_duration = work_duration;
        self.short_break_duration = short_break_duration;
        self.long_break_duration = long_break_duration;

        Ok(())
    }

    pub fn update_cycle_length(
        &mut self,
        sessions_until_long_break: u8,
    ) -> Result<()> {
        Self::validate_session_count(sessions_until_long_break)?;
        self.sessions_until_long_break = sessions_until_long_break;
        Ok(())
    }

    fn validate_durations(
        work: &Duration,
        short_break: &Duration,
        long_break: &Duration,
    ) -> Result<()> {
        if work.as_secs() < 60 || work.as_secs() > 3600 {
            return Err(Error::InvalidDuration {
                duration: work.as_secs() as u32,
            });
        }

        if short_break.as_secs() < 30 || short_break.as_secs() > 1800 {
            return Err(Error::InvalidDuration {
                duration: short_break.as_secs() as u32,
            });
        }

        if long_break.as_secs() < 300 || long_break.as_secs() > 3600 {
            return Err(Error::InvalidDuration {
                duration: long_break.as_secs() as u32,
            });
        }

        Ok(())
    }

    fn validate_session_count(count: u8) -> Result<()> {
        if count == 0 || count > 10 {
            return Err(Error::InvalidSessionCount { count });
        }
        Ok(())
    }

    fn validate_max_sessions(count: u8) -> Result<()> {
        if count == 0 || count > 10 {
            return Err(Error::InvalidSessionCount { count });
        }
        Ok(())
    }
}
