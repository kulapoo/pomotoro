use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::core::entities::{TaskId, TaskConfig, AudioConfig, TaskStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub current_sessions: u8,
    pub tags: Vec<String>,
    pub config: TaskConfig,
    pub audio_config: AudioConfig,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: TaskStatus,
}

impl Task {
    pub fn new_default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Focus Session".to_string(),
            description: Some("Default pomodoro task for focused work".to_string()),
            max_sessions: 4,
            current_sessions: 0,
            tags: vec!["focus".to_string()],
            config: TaskConfig::default(),
            audio_config: AudioConfig::default(),
            created_at: chrono::Utc::now(),
            completed_at: None,
            status: TaskStatus::Active,
        }
    }

    pub fn new(name: String, max_sessions: u8) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            max_sessions,
            current_sessions: 0,
            tags: Vec::new(),
            config: TaskConfig::default(),
            audio_config: AudioConfig::default(),
            created_at: chrono::Utc::now(),
            completed_at: None,
            status: TaskStatus::Queued,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_config(mut self, config: TaskConfig) -> Self {
        self.config = config;
        self
    }

    #[allow(dead_code)]
    pub fn with_sessions(mut self, sessions: u32) -> Self {
        self.current_sessions = sessions.min(u8::MAX as u32) as u8;
        self
    }

    #[allow(dead_code)]
    pub fn is_completed(&self) -> bool {
        self.current_sessions >= self.max_sessions || self.status == TaskStatus::Completed
    }

    pub fn increment_session(&mut self) {
        if self.current_sessions < self.max_sessions {
            self.current_sessions += 1;
            if self.current_sessions >= self.max_sessions {
                self.status = TaskStatus::Completed;
                self.completed_at = Some(chrono::Utc::now());
            }
        }
    }

    pub fn reset_sessions(&mut self) {
        self.current_sessions = 0;
        self.status = TaskStatus::Active;
        self.completed_at = None;
    }

    #[allow(dead_code)]
    pub fn get_progress_ratio(&self) -> f64 {
        if self.max_sessions == 0 {
            return 1.0;
        }
        self.current_sessions as f64 / self.max_sessions as f64
    }

    #[allow(dead_code)]
    pub fn get_remaining_sessions(&self) -> u8 {
        self.max_sessions.saturating_sub(self.current_sessions)
    }
}