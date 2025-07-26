use serde::{Deserialize, Serialize};
use pomotoro_domain::{Config, Result};
use crate::infrastructure::persistence::task_config_dto::TaskConfigDto;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfigDto {
    pub work_notification_sound: Option<String>,
    pub break_notification_sound: Option<String>,
    pub background_sound: Option<String>,
    pub volume: f32,
    pub enable_background_audio: bool,
    pub muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfigDto {
    pub task_cycling_behavior: String, // TaskCyclingBehavior serialized as string
    pub auto_start_breaks: bool,
    pub auto_start_work_after_break: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfigDto {
    pub enable_desktop_notifications: bool,
    pub enable_sound_notifications: bool,
    pub show_phase_transition_notifications: bool,
    pub show_task_completion_notifications: bool,
    pub notification_position: String, // NotificationPosition serialized as string
    pub auto_dismiss_delay_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfigDto {
    pub theme: String, // Theme serialized as string
    pub show_seconds_in_display: bool,
    pub always_on_top: bool,
    pub compact_mode: bool,
    pub show_task_list_sidebar: bool,
    pub animate_progress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDto {
    pub task: TaskConfigDto,
    pub audio: AudioConfigDto,
    pub general: GeneralConfigDto,
    pub notification: NotificationConfigDto,
    pub appearance: AppearanceConfigDto,
}

impl From<Config> for ConfigDto {
    fn from(config: Config) -> Self {
        use pomotoro_domain::{TaskCyclingBehavior, NotificationPosition, Theme};
        
        Self {
            task: TaskConfigDto::from(config.task_defaults),
            audio: AudioConfigDto {
                work_notification_sound: config.audio.work_notification_sound,
                break_notification_sound: config.audio.break_notification_sound,
                background_sound: config.audio.background_sound,
                volume: config.audio.volume,
                enable_background_audio: config.audio.enable_background_audio,
                muted: config.audio.muted,
            },
            general: GeneralConfigDto {
                task_cycling_behavior: match config.general.task_cycling_behavior {
                    TaskCyclingBehavior::Manual => "Manual".to_string(),
                    TaskCyclingBehavior::AutoAdvance => "AutoAdvance".to_string(),
                    TaskCyclingBehavior::RoundRobin => "RoundRobin".to_string(),
                },
                auto_start_breaks: config.general.auto_start_breaks,
                auto_start_work_after_break: config.general.auto_start_work_after_break,
                minimize_to_tray: config.general.minimize_to_tray,
                start_minimized: config.general.start_minimized,
            },
            notification: NotificationConfigDto {
                enable_desktop_notifications: config.notification.enable_desktop_notifications,
                enable_sound_notifications: config.notification.enable_sound_notifications,
                show_phase_transition_notifications: config.notification.show_phase_transition_notifications,
                show_task_completion_notifications: config.notification.show_task_completion_notifications,
                notification_position: match config.notification.notification_position {
                    NotificationPosition::TopRight => "TopRight".to_string(),
                    NotificationPosition::TopLeft => "TopLeft".to_string(),
                    NotificationPosition::BottomRight => "BottomRight".to_string(),
                    NotificationPosition::BottomLeft => "BottomLeft".to_string(),
                    NotificationPosition::Center => "Center".to_string(),
                },
                auto_dismiss_delay_seconds: config.notification.auto_dismiss_delay_seconds,
            },
            appearance: AppearanceConfigDto {
                theme: match config.appearance.theme {
                    Theme::Light => "Light".to_string(),
                    Theme::Dark => "Dark".to_string(),
                    Theme::System => "System".to_string(),
                },
                show_seconds_in_display: config.appearance.show_seconds_in_display,
                always_on_top: config.appearance.always_on_top,
                compact_mode: config.appearance.compact_mode,
                show_task_list_sidebar: config.appearance.show_task_list_sidebar,
                animate_progress: config.appearance.animate_progress,
            },
        }
    }
}

impl TryFrom<ConfigDto> for Config {
    type Error = pomotoro_domain::Error;
    
    fn try_from(dto: ConfigDto) -> Result<Self> {
        use pomotoro_domain::{TaskDefaults, AudioConfig, GeneralConfig, NotificationConfig, AppearanceConfig};
        use pomotoro_domain::{TaskCyclingBehavior, NotificationPosition, Theme, Error};
        
        let task_config = TaskDefaults::try_from(dto.task)?;
        
        let audio_config = AudioConfig {
            work_notification_sound: dto.audio.work_notification_sound,
            break_notification_sound: dto.audio.break_notification_sound,
            background_sound: dto.audio.background_sound,
            volume: dto.audio.volume,
            enable_background_audio: dto.audio.enable_background_audio,
            muted: dto.audio.muted,
        };
        
        let task_cycling_behavior = match dto.general.task_cycling_behavior.as_str() {
            "Manual" => TaskCyclingBehavior::Manual,
            "AutoAdvance" => TaskCyclingBehavior::AutoAdvance,
            "RoundRobin" => TaskCyclingBehavior::RoundRobin,
            _ => return Err(Error::ConfigurationError { 
                message: format!("Invalid task cycling behavior: {}", dto.general.task_cycling_behavior) 
            }),
        };
        
        let general_config = GeneralConfig {
            task_cycling_behavior,
            auto_start_breaks: dto.general.auto_start_breaks,
            auto_start_work_after_break: dto.general.auto_start_work_after_break,
            minimize_to_tray: dto.general.minimize_to_tray,
            start_minimized: dto.general.start_minimized,
        };
        
        let notification_position = match dto.notification.notification_position.as_str() {
            "TopRight" => NotificationPosition::TopRight,
            "TopLeft" => NotificationPosition::TopLeft,
            "BottomRight" => NotificationPosition::BottomRight,
            "BottomLeft" => NotificationPosition::BottomLeft,
            "Center" => NotificationPosition::Center,
            _ => return Err(Error::ConfigurationError { 
                message: format!("Invalid notification position: {}", dto.notification.notification_position) 
            }),
        };
        
        let notification_config = NotificationConfig {
            enable_desktop_notifications: dto.notification.enable_desktop_notifications,
            enable_sound_notifications: dto.notification.enable_sound_notifications,
            show_phase_transition_notifications: dto.notification.show_phase_transition_notifications,
            show_task_completion_notifications: dto.notification.show_task_completion_notifications,
            notification_position,
            auto_dismiss_delay_seconds: dto.notification.auto_dismiss_delay_seconds,
        };
        
        let theme = match dto.appearance.theme.as_str() {
            "Light" => Theme::Light,
            "Dark" => Theme::Dark,
            "System" => Theme::System,
            _ => return Err(Error::ConfigurationError { 
                message: format!("Invalid theme: {}", dto.appearance.theme) 
            }),
        };
        
        let appearance_config = AppearanceConfig {
            theme,
            show_seconds_in_display: dto.appearance.show_seconds_in_display,
            always_on_top: dto.appearance.always_on_top,
            compact_mode: dto.appearance.compact_mode,
            show_task_list_sidebar: dto.appearance.show_task_list_sidebar,
            animate_progress: dto.appearance.animate_progress,
        };
        
        Ok(Config {
            task_defaults: task_config,
            audio: audio_config,
            general: general_config,
            notification: notification_config,
            appearance: appearance_config,
        })
    }
}