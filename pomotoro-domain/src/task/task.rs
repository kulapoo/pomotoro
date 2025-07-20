use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::{TaskId, TaskStatus, TaskConfig, AudioConfig, Error, Result};

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
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: TaskStatus,
}

impl Task {
    pub fn new_default() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            name: "Focus Session".to_string(),
            description: Some("Default pomodoro task for focused work".to_string()),
            max_sessions: 4,
            current_sessions: 0,
            tags: vec!["focus".to_string()],
            config: TaskConfig::default(),
            audio_config: AudioConfig::default(),
            created_at: Utc::now(),
            completed_at: None,
            status: TaskStatus::Active,
        }
    }

    pub fn new(name: String, max_sessions: u8) -> Result<Self> {
        if name.trim().is_empty() {
            return Err(Error::InvalidSessionCount { count: 0 });
        }
        
        if max_sessions == 0 {
            return Err(Error::InvalidSessionCount { count: max_sessions });
        }

        Ok(Self {
            id: uuid::Uuid::new_v4(),
            name: name.trim().to_string(),
            description: None,
            max_sessions,
            current_sessions: 0,
            tags: Vec::new(),
            config: TaskConfig::default(),
            audio_config: AudioConfig::default(),
            created_at: Utc::now(),
            completed_at: None,
            status: TaskStatus::Queued,
        })
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_config(mut self, config: TaskConfig) -> Result<Self> {
        config.validate()?;
        self.config = config;
        Ok(self)
    }

    pub fn with_audio_config(mut self, audio_config: AudioConfig) -> Result<Self> {
        audio_config.validate()?;
        self.audio_config = audio_config;
        Ok(self)
    }

    pub fn is_completed(&self) -> bool {
        self.current_sessions >= self.max_sessions || self.status == TaskStatus::Completed
    }

    pub fn increment_session(&mut self) -> Result<()> {
        if self.is_completed() {
            return Err(Error::TaskAlreadyCompleted);
        }

        self.current_sessions += 1;
        if self.current_sessions >= self.max_sessions {
            self.status = TaskStatus::Completed;
            self.completed_at = Some(Utc::now());
        }
        
        Ok(())
    }

    pub fn reset_sessions(&mut self) {
        self.current_sessions = 0;
        self.status = TaskStatus::Active;
        self.completed_at = None;
    }

    pub fn get_progress_ratio(&self) -> f64 {
        if self.max_sessions == 0 {
            return 1.0;
        }
        self.current_sessions as f64 / self.max_sessions as f64
    }

    pub fn get_remaining_sessions(&self) -> u8 {
        self.max_sessions.saturating_sub(self.current_sessions)
    }

    pub fn pause(&mut self) -> Result<()> {
        if self.status == TaskStatus::Completed {
            return Err(Error::TaskAlreadyCompleted);
        }
        self.status = TaskStatus::Paused;
        Ok(())
    }

    pub fn activate(&mut self) -> Result<()> {
        if self.status == TaskStatus::Completed {
            return Err(Error::TaskAlreadyCompleted);
        }
        self.status = TaskStatus::Active;
        Ok(())
    }

    pub fn queue(&mut self) -> Result<()> {
        if self.status == TaskStatus::Completed {
            return Err(Error::TaskAlreadyCompleted);
        }
        self.status = TaskStatus::Queued;
        Ok(())
    }
}