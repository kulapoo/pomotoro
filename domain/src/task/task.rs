use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{AudioConfig, Error, Result};
use super::{id::Id, status::Status, config::Config};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub current_sessions: u8,
    pub tags: Vec<String>,
    pub config: Config,
    pub audio_config: AudioConfig,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: Status,
    pub default: bool,
}

impl Task {
    pub fn new_default() -> Result<Self> {
        use super::builder::Builder;

        Builder::default_task().build()
    }

    pub fn new(name: String, max_sessions: u8) -> Result<Self> {
        use super::builder::Builder;

        Builder::with_name_and_sessions(name, max_sessions).build()
    }

    pub fn new_with_defaults(
        name: String,
        max_sessions: u8,
        defaults: &crate::TaskDefaults,
    ) -> Result<Self> {
        use super::builder::Builder;

        Builder::with_name_and_sessions(name, max_sessions).build_with_defaults(defaults)
    }

    pub fn is_completed(&self) -> bool {
        self.current_sessions >= self.max_sessions || self.status == Status::Completed
    }

    pub fn increment_session(&mut self) -> Result<()> {
        if self.is_completed() {
            return Err(Error::TaskAlreadyCompleted);
        }

        self.current_sessions += 1;
        if self.current_sessions >= self.max_sessions {
            self.status = Status::Completed;
            self.completed_at = Some(Utc::now());
        }

        Ok(())
    }

    pub fn reset_sessions(&mut self) {
        self.current_sessions = 0;
        self.status = Status::Active;
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
        if self.status == Status::Completed {
            return Err(Error::TaskAlreadyCompleted);
        }
        self.status = Status::Paused;
        Ok(())
    }

    pub fn activate(&mut self) -> Result<()> {
        if self.status == Status::Completed {
            return Err(Error::TaskAlreadyCompleted);
        }
        self.status = Status::Active;
        Ok(())
    }

    pub fn queue(&mut self) -> Result<()> {
        if self.status == Status::Completed {
            return Err(Error::TaskAlreadyCompleted);
        }
        self.status = Status::Queued;
        Ok(())
    }

    pub fn is_default(&self) -> bool {
        self.default
    }

    pub fn set_as_default(&mut self) {
        self.default = true;
    }

    pub fn unset_as_default(&mut self) {
        self.default = false;
    }
}
