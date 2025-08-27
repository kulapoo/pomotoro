use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use domain::{Task, TaskId, TaskStatus};
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
            settings: domain::TaskSettings::default(),
            audio_config: domain::AudioConfig::default(),
        })
    }
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::timer_state)]
pub struct TimerStateDb {
    pub id: i32,
    pub timer_config: String, // JSON object as string
    pub current_phase: String,
    pub remaining_seconds: i32,
    pub is_running: bool,
    pub current_task_id: Option<String>,
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