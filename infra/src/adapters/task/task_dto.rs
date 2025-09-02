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
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String, // TaskStatus serialized as string
    pub default: bool,
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
            settings: Some(task.config),
            config: None, // Only used for backward compatibility during deserialization
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
        use domain::Error;

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
        let config = if let Some(settings) = dto.settings {
            settings
        } else {
            // Neither settings nor legacy config found, use defaults
            Config::default()
        };

        Ok(Task {
            id: task_id,
            timer_id: domain::TimerId::new(),
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
