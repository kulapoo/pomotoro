use crate::{AudioConfig, NotificationConfig, Result};
use crate::shared_kernel::optional_duration_serde;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskSettings {
    pub use_global_settings: bool,
    #[serde(default)]
    pub max_sessions: Option<u8>,
    #[serde(with = "optional_duration_serde", skip_serializing_if = "Option::is_none", default)]
    pub work_duration: Option<Duration>,
    #[serde(with = "optional_duration_serde", skip_serializing_if = "Option::is_none", default)]
    pub short_break_duration: Option<Duration>,
    #[serde(with = "optional_duration_serde", skip_serializing_if = "Option::is_none", default)]
    pub long_break_duration: Option<Duration>,
    #[serde(default)]
    pub sessions_until_long_break: Option<u8>,
    #[serde(default)]
    pub enable_screen_blocking: Option<bool>,
    #[serde(default)]
    pub audio_config: Option<AudioConfig>,
    #[serde(default)]
    pub notification_config: Option<NotificationConfig>,
}

impl Default for TaskSettings {
    fn default() -> Self {
        Self {
            use_global_settings: true,
            max_sessions: None,
            work_duration: None,
            short_break_duration: None,
            long_break_duration: None,
            sessions_until_long_break: None,
            enable_screen_blocking: None,
            audio_config: None,
            notification_config: None,
        }
    }
}

impl TaskSettings {
    pub fn new_with_global_defaults() -> Self {
        Self::default()
    }

    pub fn new_with_custom_settings(
        max_sessions: u8,
        work_duration: Duration,
        short_break_duration: Duration,
        long_break_duration: Duration,
        sessions_until_long_break: u8,
        enable_screen_blocking: bool,
        audio_config: AudioConfig,
        notification_config: NotificationConfig,
    ) -> Result<Self> {
        // Validate directly here instead of using Config
        Self::validate_durations(&work_duration, &short_break_duration, &long_break_duration)?;
        Self::validate_session_count(sessions_until_long_break)?;
        Self::validate_session_count(max_sessions)?;

        audio_config.validate()?;
        notification_config.validate()?;

        Ok(Self {
            use_global_settings: false,
            max_sessions: Some(max_sessions),
            work_duration: Some(work_duration),
            short_break_duration: Some(short_break_duration),
            long_break_duration: Some(long_break_duration),
            sessions_until_long_break: Some(sessions_until_long_break),
            enable_screen_blocking: Some(enable_screen_blocking),
            audio_config: Some(audio_config),
            notification_config: Some(notification_config),
        })
    }

    pub fn to_effective_settings(&self) -> EffectiveSettings {
        EffectiveSettings {
            max_sessions: self.max_sessions.unwrap_or(4),
            work_duration: self.work_duration.unwrap_or(Duration::from_secs(25 * 60)),
            short_break_duration: self.short_break_duration.unwrap_or(Duration::from_secs(5 * 60)),
            long_break_duration: self.long_break_duration.unwrap_or(Duration::from_secs(15 * 60)),
            sessions_until_long_break: self.sessions_until_long_break.unwrap_or(4),
            enable_screen_blocking: self.enable_screen_blocking.unwrap_or(false),
            audio_config: self.audio_config.clone().unwrap_or_default(),
            notification_config: self.notification_config.clone().unwrap_or_default(),
        }
    }

    pub fn reset_to_global(&mut self) {
        *self = Self::default();
    }

    pub fn has_custom_settings(&self) -> bool {
        !self.use_global_settings
    }

    pub fn update_timer_settings(
        &mut self,
        work_duration: Option<Duration>,
        short_break_duration: Option<Duration>,
        long_break_duration: Option<Duration>,
        sessions_until_long_break: Option<u8>,
        enable_screen_blocking: Option<bool>,
    ) -> Result<()> {
        let test_work = work_duration.or(self.work_duration).unwrap_or(Duration::from_secs(25 * 60));
        let test_short = short_break_duration.or(self.short_break_duration).unwrap_or(Duration::from_secs(5 * 60));
        let test_long = long_break_duration.or(self.long_break_duration).unwrap_or(Duration::from_secs(15 * 60));
        let test_sessions = sessions_until_long_break.or(self.sessions_until_long_break).unwrap_or(4);

        // Validate the new values
        Self::validate_durations(&test_work, &test_short, &test_long)?;
        Self::validate_session_count(test_sessions)?;

        if work_duration.is_some() || short_break_duration.is_some() ||
           long_break_duration.is_some() || sessions_until_long_break.is_some() ||
           enable_screen_blocking.is_some() {
            self.use_global_settings = false;
        }

        if let Some(duration) = work_duration {
            self.work_duration = Some(duration);
        }
        if let Some(duration) = short_break_duration {
            self.short_break_duration = Some(duration);
        }
        if let Some(duration) = long_break_duration {
            self.long_break_duration = Some(duration);
        }
        if let Some(sessions) = sessions_until_long_break {
            self.sessions_until_long_break = Some(sessions);
        }
        if let Some(blocking) = enable_screen_blocking {
            self.enable_screen_blocking = Some(blocking);
        }

        Ok(())
    }

    pub fn update_audio_settings(&mut self, audio_config: AudioConfig) -> Result<()> {
        audio_config.validate()?;
        self.use_global_settings = false;
        self.audio_config = Some(audio_config);
        Ok(())
    }

    pub fn update_notification_settings(&mut self, notification_config: NotificationConfig) -> Result<()> {
        notification_config.validate()?;
        self.use_global_settings = false;
        self.notification_config = Some(notification_config);
        Ok(())
    }

    pub fn update_max_sessions(&mut self, max_sessions: u8) -> Result<()> {
        Self::validate_session_count(max_sessions)?;

        self.use_global_settings = false;
        self.max_sessions = Some(max_sessions);
        Ok(())
    }

    // Validation helpers - matching TimerConfiguration's flexible ranges
    fn validate_durations(
        work: &Duration,
        short_break: &Duration,
        long_break: &Duration,
    ) -> Result<()> {
        use crate::Error;

        // Work: 1 minute to 3 hours
        if work.as_secs() < 60 || work.as_secs() > 10800 {
            return Err(Error::InvalidDuration {
                duration: work.as_secs() as u32,
            });
        }

        // Short break: 30 seconds to 1 hour
        if short_break.as_secs() < 30 || short_break.as_secs() > 3600 {
            return Err(Error::InvalidDuration {
                duration: short_break.as_secs() as u32,
            });
        }

        // Long break: 1 minute to 2 hours
        if long_break.as_secs() < 60 || long_break.as_secs() > 7200 {
            return Err(Error::InvalidDuration {
                duration: long_break.as_secs() as u32,
            });
        }

        Ok(())
    }

    fn validate_session_count(count: u8) -> Result<()> {
        use crate::Error;

        if count == 0 || count > 20 {
            return Err(Error::InvalidSessionCount { count });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectiveSettings {
    pub max_sessions: u8,
    #[serde(with = "crate::duration_serde")]
    pub work_duration: Duration,
    #[serde(with = "crate::duration_serde")]
    pub short_break_duration: Duration,
    #[serde(with = "crate::duration_serde")]
    pub long_break_duration: Duration,
    pub sessions_until_long_break: u8,
    pub enable_screen_blocking: bool,
    pub audio_config: AudioConfig,
    pub notification_config: NotificationConfig,
}

impl EffectiveSettings {
    pub fn default_values() -> Self {
        Self {
            max_sessions: 4,
            work_duration: Duration::from_secs(25 * 60),
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            sessions_until_long_break: 4,
            enable_screen_blocking: false,
            audio_config: AudioConfig::default(),
            notification_config: NotificationConfig::default(),
        }
    }

    /// Convert EffectiveSettings to TimerConfiguration
    pub fn to_timer_configuration(&self) -> Result<crate::TimerConfiguration> {
        crate::TimerConfiguration::new(
            self.work_duration,
            self.short_break_duration,
            self.long_break_duration,
            self.sessions_until_long_break,
        )
    }

    // Methods to access timing information (replacing what Config provided)
    pub fn total_cycle_duration(&self) -> Duration {
        let work_time = self.work_duration * self.sessions_until_long_break as u32;
        let short_breaks = self.short_break_duration * (self.sessions_until_long_break - 1) as u32;
        work_time + short_breaks + self.long_break_duration
    }

    pub fn is_intensive_schedule(&self) -> bool {
        self.work_duration >= Duration::from_secs(45 * 60)
            && self.sessions_until_long_break >= 6
    }

    pub fn estimated_completion_time(&self, remaining_sessions: u8) -> Duration {
        if remaining_sessions == 0 {
            return Duration::ZERO;
        }

        let work_time = self.work_duration * remaining_sessions as u32;
        let break_time = self.short_break_duration * (remaining_sessions - 1) as u32;
        work_time + break_time
    }
}