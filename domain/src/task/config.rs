use crate::{Error, Result, duration_serde};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(with = "duration_serde")]
    pub work_duration: Duration,
    #[serde(with = "duration_serde")]
    pub short_break_duration: Duration,
    #[serde(with = "duration_serde")]
    pub long_break_duration: Duration,
    pub sessions_until_long_break: u8,
    pub enable_screen_blocking: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new(
            Duration::from_secs(25 * 60),
            Duration::from_secs(5 * 60),
            Duration::from_secs(15 * 60),
            4,
            false,
        )
        .expect("Default values should always be valid")
    }
}

impl Config {
    // Constructor with validation - only way to create Config
    pub fn new(
        work_duration: Duration,
        short_break_duration: Duration,
        long_break_duration: Duration,
        sessions_until_long_break: u8,
        enable_screen_blocking: bool,
    ) -> Result<Self> {
        // Validate business rules before construction
        Self::validate_durations(
            &work_duration,
            &short_break_duration,
            &long_break_duration,
        )?;
        Self::validate_session_count(sessions_until_long_break)?;

        Ok(Self {
            work_duration,
            short_break_duration,
            long_break_duration,
            sessions_until_long_break,
            enable_screen_blocking,
        })
    }

    // Immutable accessors only
    pub fn work_duration(&self) -> Duration {
        self.work_duration
    }
    pub fn short_break_duration(&self) -> Duration {
        self.short_break_duration
    }
    pub fn long_break_duration(&self) -> Duration {
        self.long_break_duration
    }
    pub fn sessions_until_long_break(&self) -> u8 {
        self.sessions_until_long_break
    }
    pub fn enable_screen_blocking(&self) -> bool {
        self.enable_screen_blocking
    }

    // Rich domain behavior methods
    pub fn total_cycle_duration(&self) -> Duration {
        let work_time =
            self.work_duration * self.sessions_until_long_break as u32;
        let short_breaks = self.short_break_duration
            * (self.sessions_until_long_break - 1) as u32;
        work_time + short_breaks + self.long_break_duration
    }

    pub fn is_intensive_schedule(&self) -> bool {
        self.work_duration >= Duration::from_secs(45 * 60)
            && self.sessions_until_long_break >= 6
    }

    pub fn estimated_completion_time(
        &self,
        remaining_sessions: u8,
    ) -> Duration {
        if remaining_sessions == 0 {
            return Duration::ZERO;
        }

        let work_time = self.work_duration * remaining_sessions as u32;
        let break_time =
            self.short_break_duration * (remaining_sessions - 1) as u32;
        work_time + break_time
    }

    // Private validation helpers
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
}
