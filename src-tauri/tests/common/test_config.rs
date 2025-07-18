use pomotoro_lib::config::models::*;
use pomotoro_lib::config::repository::*;
use pomotoro_lib::task::models::{TaskConfig, AudioConfig};
use std::sync::RwLock;
use std::time::Duration;

pub struct TestConfigRepository {
    config: RwLock<GlobalConfig>,
}

impl TestConfigRepository {
    pub fn new() -> Self {
        Self {
            config: RwLock::new(GlobalConfig::default()),
        }
    }

    pub fn with_config(config: GlobalConfig) -> Self {
        Self {
            config: RwLock::new(config),
        }
    }

    pub fn reset_to_defaults(&self) {
        let mut config = self.config.write().unwrap();
        *config = GlobalConfig::default();
    }
}

impl pomotoro_lib::config::repository::ConfigRepo for TestConfigRepository {
    fn get_config(&self) -> Result<GlobalConfig, ConfigError> {
        let config = self.config.read().map_err(|_| ConfigError::InvalidConfig)?;
        Ok(config.clone())
    }

    fn save_config(&self, config: &GlobalConfig) -> Result<(), ConfigError> {
        let mut stored_config = self.config.write().map_err(|_| ConfigError::InvalidConfig)?;
        *stored_config = config.clone();
        Ok(())
    }

    fn reset_to_defaults(&self) -> Result<GlobalConfig, ConfigError> {
        let mut config = self.config.write().map_err(|_| ConfigError::InvalidConfig)?;
        *config = GlobalConfig::default();
        Ok(config.clone())
    }
}

pub struct TestConfigBuilder {
    config: GlobalConfig,
}

impl TestConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: GlobalConfig::default(),
        }
    }

    pub fn with_work_duration(mut self, duration: Duration) -> Self {
        self.config.default_task_config.work_duration = duration;
        self
    }

    pub fn with_short_break_duration(mut self, duration: Duration) -> Self {
        self.config.default_task_config.short_break_duration = duration;
        self
    }

    pub fn with_long_break_duration(mut self, duration: Duration) -> Self {
        self.config.default_task_config.long_break_duration = duration;
        self
    }

    pub fn with_sessions_until_long_break(mut self, sessions: u8) -> Self {
        self.config.default_task_config.sessions_until_long_break = sessions;
        self
    }

    pub fn with_screen_blocking(mut self, enabled: bool) -> Self {
        self.config.default_task_config.enable_screen_blocking = enabled;
        self
    }

    pub fn with_audio_volume(mut self, volume: f32) -> Self {
        self.config.default_audio_config.volume = volume;
        self
    }

    pub fn with_background_audio(mut self, enabled: bool) -> Self {
        self.config.default_audio_config.enable_background_audio = enabled;
        self
    }

    pub fn with_muted_audio(mut self, muted: bool) -> Self {
        self.config.default_audio_config.muted = muted;
        self
    }

    pub fn with_task_cycling(mut self, behavior: TaskCyclingBehavior) -> Self {
        self.config.app_preferences.task_cycling_behavior = behavior;
        self
    }

    pub fn with_auto_start_work_after_break(mut self, enabled: bool) -> Self {
        self.config.app_preferences.auto_start_work_after_break = enabled;
        self
    }

    pub fn with_auto_start_breaks(mut self, enabled: bool) -> Self {
        self.config.app_preferences.auto_start_breaks = enabled;
        self
    }

    pub fn with_desktop_notifications(mut self, enabled: bool) -> Self {
        self.config.notification_preferences.enable_desktop_notifications = enabled;
        self
    }

    pub fn with_notification_sounds(mut self, enabled: bool) -> Self {
        self.config.notification_preferences.enable_sound_notifications = enabled;
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.config.ui_preferences.theme = theme;
        self
    }

    pub fn with_always_on_top(mut self, enabled: bool) -> Self {
        self.config.ui_preferences.always_on_top = enabled;
        self
    }

    pub fn with_show_seconds(mut self, enabled: bool) -> Self {
        self.config.ui_preferences.show_seconds_in_display = enabled;
        self
    }

    pub fn with_compact_mode(mut self, enabled: bool) -> Self {
        self.config.ui_preferences.compact_mode = enabled;
        self
    }

    pub fn build(self) -> GlobalConfig {
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
    pub fn create_fast_config() -> GlobalConfig {
        TestConfigBuilder::new()
            .with_work_duration(Duration::from_secs(5))
            .with_short_break_duration(Duration::from_secs(2))
            .with_long_break_duration(Duration::from_secs(3))
            .with_sessions_until_long_break(2)
            .build()
    }

    pub fn create_slow_config() -> GlobalConfig {
        TestConfigBuilder::new()
            .with_work_duration(Duration::from_secs(60 * 60)) // 1 hour
            .with_short_break_duration(Duration::from_secs(30 * 60)) // 30 minutes
            .with_long_break_duration(Duration::from_secs(45 * 60)) // 45 minutes
            .with_sessions_until_long_break(6)
            .build()
    }

    pub fn create_silent_config() -> GlobalConfig {
        TestConfigBuilder::new()
            .with_muted_audio(true)
            .with_background_audio(false)
            .with_notification_sounds(false)
            .with_desktop_notifications(false)
            .build()
    }

    pub fn create_auto_advance_config() -> GlobalConfig {
        TestConfigBuilder::new()
            .with_task_cycling(TaskCyclingBehavior::AutoAdvance)
            .with_auto_start_work_after_break(true)
            .with_auto_start_breaks(true)
            .build()
    }

    pub fn create_minimal_ui_config() -> GlobalConfig {
        TestConfigBuilder::new()
            .with_compact_mode(true)
            .with_show_seconds(false)
            .with_theme(Theme::Dark)
            .build()
    }

    pub fn assert_config_equals(actual: &GlobalConfig, expected: &GlobalConfig) {
        assert_eq!(actual.default_task_config.work_duration, expected.default_task_config.work_duration);
        assert_eq!(actual.default_task_config.short_break_duration, expected.default_task_config.short_break_duration);
        assert_eq!(actual.default_task_config.long_break_duration, expected.default_task_config.long_break_duration);
        assert_eq!(actual.default_task_config.sessions_until_long_break, expected.default_task_config.sessions_until_long_break);
        assert_eq!(actual.default_task_config.enable_screen_blocking, expected.default_task_config.enable_screen_blocking);

        assert_eq!(actual.default_audio_config.volume, expected.default_audio_config.volume);
        assert_eq!(actual.default_audio_config.enable_background_audio, expected.default_audio_config.enable_background_audio);
        assert_eq!(actual.default_audio_config.muted, expected.default_audio_config.muted);

        assert_eq!(actual.app_preferences.task_cycling_behavior, expected.app_preferences.task_cycling_behavior);
        assert_eq!(actual.app_preferences.auto_start_work_after_break, expected.app_preferences.auto_start_work_after_break);
        assert_eq!(actual.app_preferences.auto_start_breaks, expected.app_preferences.auto_start_breaks);

        assert_eq!(actual.ui_preferences.theme, expected.ui_preferences.theme);
        assert_eq!(actual.ui_preferences.always_on_top, expected.ui_preferences.always_on_top);
        assert_eq!(actual.ui_preferences.compact_mode, expected.ui_preferences.compact_mode);
        assert_eq!(actual.ui_preferences.show_seconds_in_display, expected.ui_preferences.show_seconds_in_display);
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