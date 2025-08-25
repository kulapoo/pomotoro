use domain::AudioConfig;

pub struct AudioConfigBuilder {
    config: AudioConfig,
}

impl AudioConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: AudioConfig::default(),
        }
    }

    pub fn with_work_notification_sound(
        mut self,
        sound: Option<String>,
    ) -> Self {
        self.config.work_notification_sound = sound;
        self
    }

    pub fn with_break_notification_sound(
        mut self,
        sound: Option<String>,
    ) -> Self {
        self.config.break_notification_sound = sound;
        self
    }

    pub fn with_background_sound(mut self, sound: Option<String>) -> Self {
        self.config.background_sound = sound;
        self
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.config.volume = volume;
        self
    }

    pub fn with_background_audio_enabled(mut self, enabled: bool) -> Self {
        self.config.enable_background_audio = enabled;
        self
    }

    pub fn silent(mut self) -> Self {
        self.config.work_notification_sound = None;
        self.config.break_notification_sound = None;
        self.config.background_sound = None;
        self.config.volume = 0.0;
        self.config.enable_background_audio = false;
        self.config.muted = true;
        self
    }

    pub fn with_custom_sounds(mut self) -> Self {
        self.config.work_notification_sound =
            Some("custom-work-sound".to_string());
        self.config.break_notification_sound =
            Some("custom-break-sound".to_string());
        self.config.background_sound = Some("custom-background".to_string());
        self.config.volume = 0.6;
        self.config.enable_background_audio = true;
        self.config.muted = false;
        self
    }

    pub fn build(self) -> AudioConfig {
        self.config
    }
}

impl Default for AudioConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AudioTestAssertions;

impl AudioTestAssertions {
    pub fn assert_is_muted(config: &AudioConfig) {
        assert!(config.muted);
    }

    pub fn assert_is_unmuted(config: &AudioConfig) {
        assert!(!config.muted);
    }

    pub fn assert_has_background_audio(config: &AudioConfig) {
        assert!(config.enable_background_audio);
        assert!(config.background_sound.is_some());
    }

    pub fn assert_has_no_background_audio(config: &AudioConfig) {
        assert!(!config.enable_background_audio);
    }

    pub fn assert_volume_level(config: &AudioConfig, expected: f32) {
        assert!((config.volume - expected).abs() < f32::EPSILON);
    }

    pub fn assert_has_notification_sounds(config: &AudioConfig) {
        assert!(config.work_notification_sound.is_some());
        assert!(config.break_notification_sound.is_some());
    }

    pub fn assert_has_no_notification_sounds(config: &AudioConfig) {
        assert!(config.work_notification_sound.is_none());
        assert!(config.break_notification_sound.is_none());
    }
}
