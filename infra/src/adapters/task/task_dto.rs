use chrono::{DateTime, Utc};
use domain::{Result, Task, TaskId, Config, TaskStatus};
use serde::{Deserialize, Serialize};

/// Legacy config format for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyTaskConfigDto {
    pub work_duration: u64, // seconds
    pub short_break_duration: u64, // seconds
    pub long_break_duration: u64, // seconds
    pub sessions_until_long_break: u8,
    pub enable_screen_blocking: bool,
    pub max_sessions_default: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAudioConfigDto {
    pub work_notification_sound: Option<String>,
    pub break_notification_sound: Option<String>,
    pub background_sound: Option<String>,
    pub volume: f32,
    pub enable_background_audio: bool,
    pub muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDto {
    pub id: String, // TaskId serialized as string
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub current_sessions: u8,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<Config>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<LegacyTaskConfigDto>, // For backward compatibility
    pub audio_config: TaskAudioConfigDto,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String, // TaskStatus serialized as string
    pub default: bool,
}

impl From<Task> for TaskDto {
    fn from(task: Task) -> Self {
        let audio_config = TaskAudioConfigDto {
            work_notification_sound: task.config.audio.work_notification_sound.clone(),
            break_notification_sound: task.config.audio.break_notification_sound.clone(),
            background_sound: task.config.audio.background_sound.clone(),
            volume: task.config.audio.volume,
            enable_background_audio: task.config.audio.enable_background_audio,
            muted: task.config.audio.muted,
        };
        
        Self {
            id: task.id.to_string(),
            name: task.name,
            description: task.description,
            max_sessions: task.max_sessions,
            current_sessions: task.current_sessions,
            tags: task.tags,
            settings: Some(task.config),
            config: None, // Only used for backward compatibility during deserialization
            audio_config,
            created_at: task.created_at,
            completed_at: task.completed_at,
            status: match task.status {
                TaskStatus::Active => "Active".to_string(),
                TaskStatus::Queued => "Queued".to_string(),
                TaskStatus::Paused => "Paused".to_string(),
                TaskStatus::Completed => "Completed".to_string(),
            },
            default: task.default,
        }
    }
}

impl TryFrom<TaskDto> for Task {
    type Error = domain::Error;

    fn try_from(dto: TaskDto) -> Result<Self> {
        use domain::{AudioConfig, Error};

        let task_id = TaskId::from_string(&dto.id).map_err(|_| {
            Error::ConfigurationError {
                message: format!("Invalid task ID: {}", dto.id),
            }
        })?;


        let status = match dto.status.as_str() {
            "Active" => TaskStatus::Active,
            "Queued" => TaskStatus::Queued,
            "Paused" => TaskStatus::Paused,
            "Completed" => TaskStatus::Completed,
            _ => {
                return Err(Error::ConfigurationError {
                    message: format!("Invalid task status: {}", dto.status),
                });
            }
        };

        // Handle backward compatibility: convert legacy config to Config
        let mut config = if let Some(settings) = dto.settings {
            settings
        } else {
            // Neither settings nor legacy config found, use defaults
            Config::default()
        };

        // Set audio config from DTO
        config.audio = AudioConfig {
            work_notification_sound: dto.audio_config.work_notification_sound,
            break_notification_sound: dto.audio_config.break_notification_sound,
            background_sound: dto.audio_config.background_sound,
            volume: dto.audio_config.volume,
            enable_background_audio: dto.audio_config.enable_background_audio,
            muted: dto.audio_config.muted,
        };

        Ok(Task {
            id: task_id,
            name: dto.name,
            description: dto.description,
            max_sessions: dto.max_sessions,
            current_sessions: dto.current_sessions,
            tags: dto.tags,
            config,
            created_at: dto.created_at,
            completed_at: dto.completed_at,
            status,
            default: dto.default,
        })
    }
}
