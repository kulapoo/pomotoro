use std::time::Duration;
use pomotoro_domain::{TaskCyclingBehavior, NotificationPosition, Theme};
use pomotoro_domain::Config;

#[derive(Debug, Clone)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn from_config(config: Config) -> Self {
        Self { config }
    }

    pub fn build(self) -> Config {
        self.config
    }

    // Task Config flat setters/getters
    pub fn task_work_duration(mut self, duration: Duration) -> Self {
        self.config.task.work_duration = duration;
        self
    }

    pub fn get_task_work_duration(&self) -> Duration {
        self.config.task.work_duration
    }

    pub fn task_short_break_duration(mut self, duration: Duration) -> Self {
        self.config.task.short_break_duration = duration;
        self
    }

    pub fn get_task_short_break_duration(&self) -> Duration {
        self.config.task.short_break_duration
    }

    pub fn task_long_break_duration(mut self, duration: Duration) -> Self {
        self.config.task.long_break_duration = duration;
        self
    }

    pub fn get_task_long_break_duration(&self) -> Duration {
        self.config.task.long_break_duration
    }

    pub fn task_sessions_until_long_break(mut self, sessions: u8) -> Self {
        self.config.task.sessions_until_long_break = sessions;
        self
    }

    pub fn get_task_sessions_until_long_break(&self) -> u8 {
        self.config.task.sessions_until_long_break
    }

    pub fn task_enable_screen_blocking(mut self, enable: bool) -> Self {
        self.config.task.enable_screen_blocking = enable;
        self
    }

    pub fn get_task_enable_screen_blocking(&self) -> bool {
        self.config.task.enable_screen_blocking
    }

    // Audio Config flat setters/getters
    pub fn audio_work_notification_sound(mut self, sound: Option<String>) -> Self {
        self.config.audio.work_notification_sound = sound;
        self
    }

    pub fn get_audio_work_notification_sound(&self) -> &Option<String> {
        &self.config.audio.work_notification_sound
    }

    pub fn audio_break_notification_sound(mut self, sound: Option<String>) -> Self {
        self.config.audio.break_notification_sound = sound;
        self
    }

    pub fn get_audio_break_notification_sound(&self) -> &Option<String> {
        &self.config.audio.break_notification_sound
    }

    pub fn audio_background_sound(mut self, sound: Option<String>) -> Self {
        self.config.audio.background_sound = sound;
        self
    }

    pub fn get_audio_background_sound(&self) -> &Option<String> {
        &self.config.audio.background_sound
    }

    pub fn audio_volume(mut self, volume: f32) -> Self {
        self.config.audio.volume = volume;
        self
    }

    pub fn get_audio_volume(&self) -> f32 {
        self.config.audio.volume
    }

    pub fn audio_enable_background_audio(mut self, enable: bool) -> Self {
        self.config.audio.enable_background_audio = enable;
        self
    }

    pub fn get_audio_enable_background_audio(&self) -> bool {
        self.config.audio.enable_background_audio
    }

    pub fn audio_muted(mut self, muted: bool) -> Self {
        self.config.audio.muted = muted;
        self
    }

    pub fn get_audio_muted(&self) -> bool {
        self.config.audio.muted
    }

    // General flat setters/getters
    pub fn general_task_cycling_behavior(mut self, behavior: TaskCyclingBehavior) -> Self {
        self.config.general.task_cycling_behavior = behavior;
        self
    }

    pub fn get_general_task_cycling_behavior(&self) -> TaskCyclingBehavior {
        self.config.general.task_cycling_behavior.clone()
    }

    pub fn general_max_sessions_default(mut self, max_sessions: u8) -> Self {
        self.config.general.max_sessions_default = max_sessions;
        self
    }

    pub fn get_general_max_sessions_default(&self) -> u8 {
        self.config.general.max_sessions_default
    }

    pub fn general_auto_start_breaks(mut self, auto_start: bool) -> Self {
        self.config.general.auto_start_breaks = auto_start;
        self
    }

    pub fn get_general_auto_start_breaks(&self) -> bool {
        self.config.general.auto_start_breaks
    }

    pub fn general_auto_start_work_after_break(mut self, auto_start: bool) -> Self {
        self.config.general.auto_start_work_after_break = auto_start;
        self
    }

    pub fn get_general_auto_start_work_after_break(&self) -> bool {
        self.config.general.auto_start_work_after_break
    }

    pub fn general_minimize_to_tray(mut self, minimize: bool) -> Self {
        self.config.general.minimize_to_tray = minimize;
        self
    }

    pub fn get_general_minimize_to_tray(&self) -> bool {
        self.config.general.minimize_to_tray
    }

    pub fn general_start_minimized(mut self, start_minimized: bool) -> Self {
        self.config.general.start_minimized = start_minimized;
        self
    }

    pub fn get_general_start_minimized(&self) -> bool {
        self.config.general.start_minimized
    }

    // Notification Preferences flat setters/getters
    pub fn notification_enable_desktop_notifications(mut self, enable: bool) -> Self {
        self.config.notification.enable_desktop_notifications = enable;
        self
    }

    pub fn get_notification_enable_desktop_notifications(&self) -> bool {
        self.config.notification.enable_desktop_notifications
    }

    pub fn notification_enable_sound_notifications(mut self, enable: bool) -> Self {
        self.config.notification.enable_sound_notifications = enable;
        self
    }

    pub fn get_notification_enable_sound_notifications(&self) -> bool {
        self.config.notification.enable_sound_notifications
    }

    pub fn notification_show_phase_transition_notifications(mut self, show: bool) -> Self {
        self.config.notification.show_phase_transition_notifications = show;
        self
    }

    pub fn get_notification_show_phase_transition_notifications(&self) -> bool {
        self.config.notification.show_phase_transition_notifications
    }

    pub fn notification_show_task_completion_notifications(mut self, show: bool) -> Self {
        self.config.notification.show_task_completion_notifications = show;
        self
    }

    pub fn get_notification_show_task_completion_notifications(&self) -> bool {
        self.config.notification.show_task_completion_notifications
    }

    pub fn notification_position(mut self, position: NotificationPosition) -> Self {
        self.config.notification.notification_position = position;
        self
    }

    pub fn get_notification_position(&self) -> NotificationPosition {
        self.config.notification.notification_position.clone()
    }

    pub fn notification_auto_dismiss_delay_seconds(mut self, delay: u32) -> Self {
        self.config.notification.auto_dismiss_delay_seconds = delay;
        self
    }

    pub fn get_notification_auto_dismiss_delay_seconds(&self) -> u32 {
        self.config.notification.auto_dismiss_delay_seconds
    }

    // Appearance flat setters/getters
    pub fn appearance_theme(mut self, theme: Theme) -> Self {
        self.config.appearance.theme = theme;
        self
    }

    pub fn get_appearance_theme(&self) -> Theme {
        self.config.appearance.theme.clone()
    }

    pub fn appearance_show_seconds_in_display(mut self, show: bool) -> Self {
        self.config.appearance.show_seconds_in_display = show;
        self
    }

    pub fn get_appearance_show_seconds_in_display(&self) -> bool {
        self.config.appearance.show_seconds_in_display
    }

    pub fn appearance_always_on_top(mut self, always_on_top: bool) -> Self {
        self.config.appearance.always_on_top = always_on_top;
        self
    }

    pub fn get_appearance_always_on_top(&self) -> bool {
        self.config.appearance.always_on_top
    }

    pub fn appearance_compact_mode(mut self, compact: bool) -> Self {
        self.config.appearance.compact_mode = compact;
        self
    }

    pub fn get_appearance_compact_mode(&self) -> bool {
        self.config.appearance.compact_mode
    }

    pub fn appearance_show_task_list_sidebar(mut self, show: bool) -> Self {
        self.config.appearance.show_task_list_sidebar = show;
        self
    }

    pub fn get_appearance_show_task_list_sidebar(&self) -> bool {
        self.config.appearance.show_task_list_sidebar
    }

    pub fn appearance_animate_progress(mut self, animate: bool) -> Self {
        self.config.appearance.animate_progress = animate;
        self
    }

    pub fn get_appearance_animate_progress(&self) -> bool {
        self.config.appearance.animate_progress
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}