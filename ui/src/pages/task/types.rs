use chrono::{DateTime, Utc};
use domain::{Config, Task, TaskId, TaskStatus};
use serde::{Deserialize, Serialize};

// DTO to match backend's TaskDto
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub current_sessions: u8,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<Config>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
    pub default: bool,
}

impl TaskDto {
    pub fn to_task(&self) -> Result<Task, String> {
        let task_id = TaskId::from_string(&self.id)
            .map_err(|e| format!("Invalid task ID: {}", e))?;

        let status = match self.status.as_str() {
            "Active" | "active" => TaskStatus::Active,
            "Completed" | "completed" => TaskStatus::Completed,
            "Paused" | "paused" => TaskStatus::Paused,
            "Queued" | "queued" => TaskStatus::Queued,
            _ => TaskStatus::Queued,
        };

        let config = self.settings.clone().unwrap_or_default();

        Ok(Task {
            id: task_id,
            name: self.name.clone(),
            description: self.description.clone(),
            max_sessions: self.max_sessions,
            current_sessions: self.current_sessions,
            tags: self.tags.clone(),
            config,
            created_at: self.created_at,
            updated_at: self.created_at,
            completed_at: self.completed_at,
            status,
            default: self.default,
        })
    }
}