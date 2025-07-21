use pomotoro_domain::{Config, TaskConfig, AudioConfig};
use pomotoro_lib::config::{ConfigRepo, ConfigError};
use std::sync::RwLock;
use std::time::Duration;

pub struct TestConfigRepository {
    config: RwLock<Config>,
}

impl TestConfigRepository {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(Config::default()),
        }
    }

    pub fn with_config(config: Config) -> Self {
        Self {
            config: RwLock::new(config),
        }
    }

    pub fn reset_to_defaults(&self) {
        let mut config = self.config.write().unwrap();
        *config = Config::default();
    }
}

impl ConfigRepo for TestConfigRepository {
    fn get_config(&self) -> Result<Config, ConfigError> {
        let config = self.config.read().map_err(|_| ConfigError::InvalidConfig)?;
        Ok(config.clone())
    }

    fn save_config(&self, config: &Config) -> Result<(), ConfigError> {
        let mut stored_config = self.config.write().map_err(|_| ConfigError::InvalidConfig)?;
        *stored_config = config.clone();
        Ok(())
    }

    fn reset_to_defaults(&self) -> Result<Config, ConfigError> {
        let mut config = self.config.write().map_err(|_| ConfigError::InvalidConfig)?;
        *config = Config::default();
        Ok(config.clone())
    }
}

pub struct TestConfigBuilder {
    config: Config,
}

impl TestConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn with_work_duration(mut self, duration: Duration) -> Self {
        self.config.task_config.work_duration = duration;
        self
    }

    pub fn with_short_break_duration(mut self, duration: Duration) -> Self {
        self.config.task_config.short_break_duration = duration;
        self
    }

    pub fn with_long_break_duration(mut self, duration: Duration) -> Self {
        self.config.task_config.long_break_duration = duration;
        self
    }

    pub fn with_sessions_until_long_break(mut self, sessions: u8) -> Self {
        self.config.task_config.sessions_until_long_break = sessions;
        self
    }

    pub fn with_screen_blocking(mut self, enabled: bool) -> Self {
        self.config.task_config.enable_screen_blocking = enabled;
        self
    }

    pub fn with_audio_volume(mut self, volume: f32) -> Self {
        self.config.audio_config.volume = volume;
        self
    }

    pub fn with_background_audio(mut self, enabled: bool) -> Self {
        self.config.audio_config.enable_background_audio = enabled;
        self
    }

    pub fn with_muted_audio(mut self, muted: bool) -> Self {
        self.config.audio_config.muted = muted;
        self
    }


    pub fn build(self) -> Config {
        self.config
    }
}

impl Default for TestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ConfigTestUtils;

impl ConfigTestUtils {
    pub fn create_fast_config() -> Config {
        TestConfigBuilder::new()
            .with_work_duration(Duration::from_secs(5))
            .with_short_break_duration(Duration::from_secs(2))
            .with_long_break_duration(Duration::from_secs(3))
            .with_sessions_until_long_break(2)
            .build()
    }

    pub fn create_slow_config() -> Config {
        TestConfigBuilder::new()
            .with_work_duration(Duration::from_secs(60 * 60)) // 1 hour
            .with_short_break_duration(Duration::from_secs(30 * 60)) // 30 minutes
            .with_long_break_duration(Duration::from_secs(45 * 60)) // 45 minutes
            .with_sessions_until_long_break(6)
            .build()
    }

    pub fn create_silent_config() -> Config {
        TestConfigBuilder::new()
            .with_muted_audio(true)
            .with_background_audio(false)
            .build()
    }

    pub fn assert_config_equals(actual: &Config, expected: &Config) {
        assert_eq!(actual.task_config.work_duration, expected.task_config.work_duration);
        assert_eq!(actual.task_config.short_break_duration, expected.task_config.short_break_duration);
        assert_eq!(actual.task_config.long_break_duration, expected.task_config.long_break_duration);
        assert_eq!(actual.task_config.sessions_until_long_break, expected.task_config.sessions_until_long_break);
        assert_eq!(actual.task_config.enable_screen_blocking, expected.task_config.enable_screen_blocking);

        assert_eq!(actual.audio_config.volume, expected.audio_config.volume);
        assert_eq!(actual.audio_config.enable_background_audio, expected.audio_config.enable_background_audio);
        assert_eq!(actual.audio_config.muted, expected.audio_config.muted);
    }

    pub fn assert_task_config_equals(actual: &TaskConfig, expected: &TaskConfig) {
        assert_eq!(actual.work_duration, expected.work_duration);
        assert_eq!(actual.short_break_duration, expected.short_break_duration);
        assert_eq!(actual.long_break_duration, expected.long_break_duration);
        assert_eq!(actual.sessions_until_long_break, expected.sessions_until_long_break);
        assert_eq!(actual.enable_screen_blocking, expected.enable_screen_blocking);
    }

    pub fn assert_audio_config_equals(actual: &AudioConfig, expected: &AudioConfig) {
        assert_eq!(actual.work_notification_sound, expected.work_notification_sound);
        assert_eq!(actual.break_notification_sound, expected.break_notification_sound);
        assert_eq!(actual.background_sound, expected.background_sound);
        assert_eq!(actual.volume, expected.volume);
        assert_eq!(actual.enable_background_audio, expected.enable_background_audio);
        assert_eq!(actual.muted, expected.muted);
    }
}