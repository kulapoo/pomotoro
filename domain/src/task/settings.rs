use crate::{AudioConfig, NotificationConfig, Result, TaskDefaults};
use crate::shared_kernel::optional_duration_serde;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskSettings {
    pub use_global_settings: bool,
    #[serde(default)]
    pub custom_max_sessions: Option<u8>,
    #[serde(with = "optional_duration_serde", skip_serializing_if = "Option::is_none", default)]
    pub custom_work_duration: Option<Duration>,
    #[serde(with = "optional_duration_serde", skip_serializing_if = "Option::is_none", default)]
    pub custom_short_break_duration: Option<Duration>,
    #[serde(with = "optional_duration_serde", skip_serializing_if = "Option::is_none", default)]
    pub custom_long_break_duration: Option<Duration>,
    #[serde(default)]
    pub custom_sessions_until_long_break: Option<u8>,
    #[serde(default)]
    pub custom_enable_screen_blocking: Option<bool>,
    #[serde(default)]
    pub custom_audio_config: Option<AudioConfig>,
    #[serde(default)]
    pub custom_notification_config: Option<NotificationConfig>,
}

impl Default for TaskSettings {
    fn default() -> Self {
        Self {
            use_global_settings: true,
            custom_max_sessions: None,
            custom_work_duration: None,
            custom_short_break_duration: None,
            custom_long_break_duration: None,
            custom_sessions_until_long_break: None,
            custom_enable_screen_blocking: None,
            custom_audio_config: None,
            custom_notification_config: None,
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
            custom_max_sessions: Some(max_sessions),
            custom_work_duration: Some(work_duration),
            custom_short_break_duration: Some(short_break_duration),
            custom_long_break_duration: Some(long_break_duration),
            custom_sessions_until_long_break: Some(sessions_until_long_break),
            custom_enable_screen_blocking: Some(enable_screen_blocking),
            custom_audio_config: Some(audio_config),
            custom_notification_config: Some(notification_config),
        })
    }

    pub fn merge_with_defaults(&self, defaults: &TaskDefaults) -> EffectiveSettings {
        if self.use_global_settings {
            EffectiveSettings::from_defaults(defaults)
        } else {
            EffectiveSettings {
                max_sessions: self.custom_max_sessions.unwrap_or(defaults.max_sessions_default),
                work_duration: self.custom_work_duration.unwrap_or(defaults.work_duration),
                short_break_duration: self.custom_short_break_duration.unwrap_or(defaults.short_break_duration),
                long_break_duration: self.custom_long_break_duration.unwrap_or(defaults.long_break_duration),
                sessions_until_long_break: self.custom_sessions_until_long_break.unwrap_or(defaults.sessions_until_long_break),
                enable_screen_blocking: self.custom_enable_screen_blocking.unwrap_or(defaults.enable_screen_blocking),
                audio_config: self.custom_audio_config.clone().unwrap_or_default(),
                notification_config: self.custom_notification_config.clone().unwrap_or_default(),
            }
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
        let test_work = work_duration.or(self.custom_work_duration).unwrap_or(Duration::from_secs(25 * 60));
        let test_short = short_break_duration.or(self.custom_short_break_duration).unwrap_or(Duration::from_secs(5 * 60));
        let test_long = long_break_duration.or(self.custom_long_break_duration).unwrap_or(Duration::from_secs(15 * 60));
        let test_sessions = sessions_until_long_break.or(self.custom_sessions_until_long_break).unwrap_or(4);

        // Validate the new values
        Self::validate_durations(&test_work, &test_short, &test_long)?;
        Self::validate_session_count(test_sessions)?;

        if work_duration.is_some() || short_break_duration.is_some() || 
           long_break_duration.is_some() || sessions_until_long_break.is_some() || 
           enable_screen_blocking.is_some() {
            self.use_global_settings = false;
        }

        if let Some(duration) = work_duration {
            self.custom_work_duration = Some(duration);
        }
        if let Some(duration) = short_break_duration {
            self.custom_short_break_duration = Some(duration);
        }
        if let Some(duration) = long_break_duration {
            self.custom_long_break_duration = Some(duration);
        }
        if let Some(sessions) = sessions_until_long_break {
            self.custom_sessions_until_long_break = Some(sessions);
        }
        if let Some(blocking) = enable_screen_blocking {
            self.custom_enable_screen_blocking = Some(blocking);
        }

        Ok(())
    }

    pub fn update_audio_settings(&mut self, audio_config: AudioConfig) -> Result<()> {
        audio_config.validate()?;
        self.use_global_settings = false;
        self.custom_audio_config = Some(audio_config);
        Ok(())
    }

    pub fn update_notification_settings(&mut self, notification_config: NotificationConfig) -> Result<()> {
        notification_config.validate()?;
        self.use_global_settings = false;
        self.custom_notification_config = Some(notification_config);
        Ok(())
    }

    pub fn update_max_sessions(&mut self, max_sessions: u8) -> Result<()> {
        Self::validate_session_count(max_sessions)?;
        
        self.use_global_settings = false;
        self.custom_max_sessions = Some(max_sessions);
        Ok(())
    }
    
    // Validation helpers
    fn validate_durations(
        work: &Duration,
        short_break: &Duration,
        long_break: &Duration,
    ) -> Result<()> {
        use crate::Error;
        
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
        use crate::Error;
        
        if count == 0 || count > 10 {
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
    pub fn from_defaults(defaults: &TaskDefaults) -> Self {
        Self {
            max_sessions: defaults.max_sessions_default,
            work_duration: defaults.work_duration,
            short_break_duration: defaults.short_break_duration,
            long_break_duration: defaults.long_break_duration,
            sessions_until_long_break: defaults.sessions_until_long_break,
            enable_screen_blocking: defaults.enable_screen_blocking,
            audio_config: AudioConfig::default(),
            notification_config: NotificationConfig::default(),
        }
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