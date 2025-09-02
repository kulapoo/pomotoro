use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use domain::{Task, TaskId, TaskStatus, Timer, TimerId, timer::{TimerState, Phase}};
use uuid::Uuid;

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::tasks)]
pub struct TaskDb {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub sessions: i32,
    pub current_sessions: i32,
    pub status: String,
    pub tags: Option<String>, // JSON array as string
    pub config: String, // JSON object with timer configuration and other settings
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Task> for TaskDb {
    fn from(task: Task) -> Self {
        let tags = if task.tags.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&task.tags).unwrap_or_default())
        };

        let config = serde_json::to_string(&task.config).unwrap_or_default();

        Self {
            id: task.id.to_string(),
            name: task.name,
            description: task.description,
            sessions: task.max_sessions as i32,
            current_sessions: task.current_sessions as i32,
            status: match task.status {
                TaskStatus::Active => "active",
                TaskStatus::Completed => "completed",
                TaskStatus::Paused => "paused",
                TaskStatus::Queued => "queued",
            }.to_string(),
            tags,
            config,
            is_default: task.default,
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.created_at.to_rfc3339(),
        }
    }
}

impl TryFrom<TaskDb> for Task {
    type Error = domain::Error;

    fn try_from(db: TaskDb) -> Result<Self, Self::Error> {
        let tags: Vec<String> = if let Some(tags_json) = db.tags {
            serde_json::from_str(&tags_json).unwrap_or_default()
        } else {
            vec![]
        };

        let status = match db.status.as_str() {
            "active" => TaskStatus::Active,
            "completed" => TaskStatus::Completed,
            "paused" => TaskStatus::Paused,
            "queued" | _ => TaskStatus::Queued,
        };

        let uuid = Uuid::parse_str(&db.id).map_err(|_e| domain::Error::SerializationError {
            message: format!("Invalid task ID: {}", db.id),
        })?;
        let task_id = TaskId::from_uuid(uuid);

        let created_at = DateTime::parse_from_rfc3339(&db.created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| domain::Error::SerializationError {
                message: format!("Invalid created_at timestamp: {}", e),
            })?;

        let config: domain::Config = serde_json::from_str(&db.config)
            .unwrap_or_else(|_| domain::Config::default());

        Ok(Task {
            id: task_id,
            name: db.name,
            description: db.description,
            max_sessions: db.sessions as u8,
            current_sessions: db.current_sessions as u8,
            status,
            tags,
            default: db.is_default,
            created_at,
            completed_at: None,
            config,
        })
    }
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::timers)]
pub struct TimerDb {
    pub id: String,
    pub active_task_id: Option<String>,
    pub current_phase: String,
    pub remaining_seconds: i32,
    pub is_running: bool,
    pub session_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::config)]
pub struct ConfigDb {
    pub id: i32,
    pub config_data: String, // JSON object as string
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::session_history)]
pub struct SessionHistoryDb {
    pub id: String,
    pub task_id: String,
    pub session_type: String,
    pub duration_seconds: i32,
    pub completed_at: String,
}

impl From<Timer> for TimerDb {
    fn from(timer: Timer) -> Self {
        let current_phase = match timer.state().phase() {
            Phase::Work => "work",
            Phase::ShortBreak => "short_break",
            Phase::LongBreak => "long_break",
        }.to_string();
        
        Self {
            id: timer.id().to_string(),
            active_task_id: timer.active_task_id().map(|id| id.to_string()),
            current_phase,
            remaining_seconds: timer.state().remaining_seconds() as i32,
            is_running: timer.state().is_running(),
            session_count: 0, // Session count is now tracked in Task
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}

impl TryFrom<TimerDb> for Timer {
    type Error = domain::Error;
    
    fn try_from(db: TimerDb) -> Result<Self, Self::Error> {
        let timer_uuid = Uuid::parse_str(&db.id).map_err(|e| domain::Error::SerializationError {
            message: format!("Invalid timer ID: {}", e),
        })?;
        let timer_id = TimerId::from_uuid(timer_uuid);
        
        let active_task_id = if let Some(task_id_str) = db.active_task_id {
            let task_uuid = Uuid::parse_str(&task_id_str).map_err(|e| domain::Error::SerializationError {
                message: format!("Invalid task ID: {}", e),
            })?;
            Some(TaskId::from_uuid(task_uuid))
        } else {
            None
        };
        
        let state = match db.current_phase.as_str() {
            "work" | "Work" if !db.is_running => TimerState::Idle,
            "work" | "Work" => TimerState::Working {
                remaining_seconds: db.remaining_seconds as u32,
            },
            "short_break" | "ShortBreak" => TimerState::ShortBreak {
                remaining_seconds: db.remaining_seconds as u32,
            },
            "long_break" | "LongBreak" => TimerState::LongBreak {
                remaining_seconds: db.remaining_seconds as u32,
            },
            _ => TimerState::Idle,
        };
        
        let mut timer = Timer::with_state(timer_id, state);
        if let Some(task_id) = active_task_id {
            timer.set_active_task(task_id);
        }
        Ok(timer)
    }
}