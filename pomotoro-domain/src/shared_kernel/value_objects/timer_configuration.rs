use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{Error, Result, Phase, duration_serde};

/// Timer configuration value object for timing-related settings.
///
/// This value object encapsulates all timing configuration needed by the Timer domain
/// without coupling to specific Task or other domain concerns. It represents the
/// timing rules for pomodoro sessions in a domain-agnostic way.
///
/// # Domain Invariants
/// - Work duration must be between 1-60 minutes
/// - Short break duration must be between 30 seconds-30 minutes  
/// - Long break duration must be between 5-60 minutes
/// - Sessions until long break must be between 1-10
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimerConfiguration {
    #[serde(with = "duration_serde")]
    pub work_duration: Duration,
    #[serde(with = "duration_serde")]
    pub short_break_duration: Duration,
    #[serde(with = "duration_serde")]
    pub long_break_duration: Duration,
    pub sessions_until_long_break: u8,
}

impl Default for TimerConfiguration {
    fn default() -> Self {
        Self {
            work_duration: Duration::from_secs(25 * 60),        // 25 minutes
            short_break_duration: Duration::from_secs(5 * 60),  // 5 minutes
            long_break_duration: Duration::from_secs(15 * 60),  // 15 minutes
            sessions_until_long_break: 4,                       // Traditional pomodoro
        }
    }
}

impl TimerConfiguration {
    /// Create a new timer configuration with validation.
    pub fn new(
        work_duration: Duration,
        short_break_duration: Duration,
        long_break_duration: Duration,
        sessions_until_long_break: u8,
    ) -> Result<Self> {
        let config = Self {
            work_duration,
            short_break_duration,
            long_break_duration,
            sessions_until_long_break,
        };
        
        config.validate()?;
        Ok(config)
    }

    /// Get the duration for a specific phase.
    pub fn get_phase_duration(&self, phase: Phase) -> Duration {
        match phase {
            Phase::Work => self.work_duration,
            Phase::ShortBreak => self.short_break_duration,
            Phase::LongBreak => self.long_break_duration,
        }
    }

    /// Get the duration for a specific phase in seconds.
    pub fn get_phase_duration_seconds(&self, phase: Phase) -> u32 {
        self.get_phase_duration(phase).as_secs() as u32
    }

    /// Validate the timer configuration invariants.
    pub fn validate(&self) -> Result<()> {
        // Work duration: 1-60 minutes
        let work_secs = self.work_duration.as_secs();
        if !(60..=3600).contains(&work_secs) {
            return Err(Error::InvalidDuration {
                duration: work_secs as u32,
            });
        }
        
        // Short break duration: 30 seconds - 30 minutes
        let short_break_secs = self.short_break_duration.as_secs();
        if !(30..=1800).contains(&short_break_secs) {
            return Err(Error::InvalidDuration {
                duration: short_break_secs as u32,
            });
        }
        
        // Long break duration: 5-60 minutes
        let long_break_secs = self.long_break_duration.as_secs();
        if !(300..=3600).contains(&long_break_secs) {
            return Err(Error::InvalidDuration {
                duration: long_break_secs as u32,
            });
        }
        
        // Sessions until long break: 1-10
        if self.sessions_until_long_break == 0 || self.sessions_until_long_break > 10 {
            return Err(Error::InvalidSessionCount {
                count: self.sessions_until_long_break,
            });
        }
        
        Ok(())
    }

    /// Create a timer configuration optimized for focused work.
    pub fn focused_work() -> Self {
        Self {
            work_duration: Duration::from_secs(50 * 60),    // 50 minutes
            short_break_duration: Duration::from_secs(10 * 60), // 10 minutes
            long_break_duration: Duration::from_secs(30 * 60),  // 30 minutes
            sessions_until_long_break: 2,
        }
    }

    /// Create a timer configuration optimized for quick tasks.
    pub fn quick_tasks() -> Self {
        Self {
            work_duration: Duration::from_secs(15 * 60),    // 15 minutes
            short_break_duration: Duration::from_secs(3 * 60),  // 3 minutes
            long_break_duration: Duration::from_secs(10 * 60),  // 10 minutes
            sessions_until_long_break: 6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_default_configuration() {
        let config = TimerConfiguration::default();
        assert_eq!(config.work_duration, Duration::from_secs(25 * 60));
        assert_eq!(config.short_break_duration, Duration::from_secs(5 * 60));
        assert_eq!(config.long_break_duration, Duration::from_secs(15 * 60));
        assert_eq!(config.sessions_until_long_break, 4);
    }

    #[test]
    fn should_validate_valid_configuration() {
        let config = TimerConfiguration::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn should_reject_invalid_work_duration() {
        let config = TimerConfiguration {
            work_duration: Duration::from_secs(30), // Too short
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn should_reject_invalid_sessions_count() {
        let config = TimerConfiguration {
            sessions_until_long_break: 0, // Invalid
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn should_get_phase_duration() {
        let config = TimerConfiguration::default();
        assert_eq!(config.get_phase_duration(Phase::Work), Duration::from_secs(25 * 60));
        assert_eq!(config.get_phase_duration(Phase::ShortBreak), Duration::from_secs(5 * 60));
        assert_eq!(config.get_phase_duration(Phase::LongBreak), Duration::from_secs(15 * 60));
    }

    #[test]
    fn should_get_phase_duration_seconds() {
        let config = TimerConfiguration::default();
        assert_eq!(config.get_phase_duration_seconds(Phase::Work), 25 * 60);
        assert_eq!(config.get_phase_duration_seconds(Phase::ShortBreak), 5 * 60);
        assert_eq!(config.get_phase_duration_seconds(Phase::LongBreak), 15 * 60);
    }

    #[test]
    fn should_create_focused_work_configuration() {
        let config = TimerConfiguration::focused_work();
        assert_eq!(config.work_duration, Duration::from_secs(50 * 60));
        assert_eq!(config.sessions_until_long_break, 2);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn should_create_quick_tasks_configuration() {
        let config = TimerConfiguration::quick_tasks();
        assert_eq!(config.work_duration, Duration::from_secs(15 * 60));
        assert_eq!(config.sessions_until_long_break, 6);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn should_serialize_and_deserialize() {
        let config = TimerConfiguration::default();
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: TimerConfiguration = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }
}