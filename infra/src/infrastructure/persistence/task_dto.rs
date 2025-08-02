use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use domain::{Task, TaskId, TaskStatus, Result};
use crate::infrastructure::persistence::task_config_dto::TaskConfigDto;

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
    pub config: TaskConfigDto,
    pub audio_config: TaskAudioConfigDto,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String, // TaskStatus serialized as string
}

impl From<Task> for TaskDto {
    fn from(task: Task) -> Self {
        Self {
            id: task.id.to_string(),
            name: task.name,
            description: task.description,
            max_sessions: task.max_sessions,
            current_sessions: task.current_sessions,
            tags: task.tags,
            config: TaskConfigDto::from(task.config),
            audio_config: TaskAudioConfigDto {
                work_notification_sound: task.audio_config.work_notification_sound,
                break_notification_sound: task.audio_config.break_notification_sound,
                background_sound: task.audio_config.background_sound,
                volume: task.audio_config.volume,
                enable_background_audio: task.audio_config.enable_background_audio,
                muted: task.audio_config.muted,
            },
            created_at: task.created_at,
            completed_at: task.completed_at,
            status: match task.status {
                TaskStatus::Active => "Active".to_string(),
                TaskStatus::Queued => "Queued".to_string(),
                TaskStatus::Paused => "Paused".to_string(),
                TaskStatus::Completed => "Completed".to_string(),
            },
        }
    }
}

impl TryFrom<TaskDto> for Task {
    type Error = domain::Error;
    
    fn try_from(dto: TaskDto) -> Result<Self> {
        use domain::{TaskConfig, AudioConfig, Error};
        
        let task_id = TaskId::from_string(&dto.id)
            .map_err(|_| Error::ConfigurationError { 
                message: format!("Invalid task ID: {}", dto.id) 
            })?;
        
        let task_config = TaskConfig::try_from(dto.config)?;
        
        let audio_config = AudioConfig {
            work_notification_sound: dto.audio_config.work_notification_sound,
            break_notification_sound: dto.audio_config.break_notification_sound,
            background_sound: dto.audio_config.background_sound,
            volume: dto.audio_config.volume,
            enable_background_audio: dto.audio_config.enable_background_audio,
            muted: dto.audio_config.muted,
        };
        
        let status = match dto.status.as_str() {
            "Active" => TaskStatus::Active,
            "Queued" => TaskStatus::Queued,
            "Paused" => TaskStatus::Paused,
            "Completed" => TaskStatus::Completed,
            _ => return Err(Error::ConfigurationError { 
                message: format!("Invalid task status: {}", dto.status) 
            }),
        };
        
        Ok(Task {
            id: task_id,
            name: dto.name,
            description: dto.description,
            max_sessions: dto.max_sessions,
            current_sessions: dto.current_sessions,
            tags: dto.tags,
            config: task_config,
            audio_config,
            created_at: dto.created_at,
            completed_at: dto.completed_at,
            status,
        })
    }
}