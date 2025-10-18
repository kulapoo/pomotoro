use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{id::Id, status::Status};
use crate::{Config, Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Id,
    pub name: String,
    pub description: Option<String>,
    pub max_sessions: u8,
    pub current_sessions: u8,
    pub tags: Vec<String>,
    pub config: Config,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: Status,
    pub default: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TaskPatch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub max_sessions: Option<u8>,
    pub tags: Option<Vec<String>>,
    pub config: Option<Config>,
    pub status: Option<Status>,
    pub default: Option<bool>,
    pub current_sessions: Option<u8>,
    pub updated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    pub fn id(&self) -> Id {
        self.id
    }

    pub fn new_default() -> Result<Self> {
        use super::builder::Builder;

        Builder::default_task().build()
    }

    pub fn new(name: String, max_sessions: u8) -> Result<Self> {
        use super::builder::Builder;

        Builder::with_name_and_sessions(name, max_sessions).build()
    }

    pub fn new_with_defaults(name: String, max_sessions: u8) -> Result<Self> {
        use super::builder::Builder;

        Builder::with_name_and_sessions(name, max_sessions).build()
    }

    pub fn is_completed(&self) -> bool {
        self.current_sessions >= self.max_sessions
            && self.status == Status::Completed
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

    pub fn with_default(mut self, default: bool) -> Self {
        self.default = default;
        self
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    pub fn reset_config(&mut self) {
        self.config = Config::default();
    }
}

impl Task {
    pub fn patch(&mut self, update: TaskPatch) {
        if let Some(name) = update.name {
            self.name = name;
        }
        if let Some(description) = update.description {
            self.description = Some(description);
        }
        if let Some(max_sessions) = update.max_sessions {
            self.max_sessions = max_sessions;
        }
        if let Some(tags) = update.tags {
            self.tags = tags;
        }
        if let Some(default) = update.default {
            self.default = default;
        }
        if let Some(updated_at) = update.updated_at {
            self.created_at = updated_at;
        }
        if let Some(status) = update.status {
            self.status = status;
        }
        if let Some(config) = update.config {
            self.config = config;
        }
        if let Some(current_sessions) = update.current_sessions {
            self.current_sessions = current_sessions;
        }
        if let Some(completed_at) = update.completed_at {
            self.completed_at = Some(completed_at);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::TaskBuilder;

    use super::*;

    fn create_test_task() -> Task {
        TaskBuilder::new()
            .id(Id::new())
            .name("Test Task".to_string())
            .description("Test description".to_string())
            .max_sessions(5)
            .current_sessions(0)
            .tags(vec!["test".to_string(), "sample".to_string()])
            .config(Config::default())
            .created_at(Utc::now())
            .completed_at(Utc::now())
            .status(Status::Active)
            .default(false)
            .build()
            .unwrap()
    }

    #[test]
    fn test_task_creation() {
        let task = create_test_task();
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.max_sessions, 5);
        assert_eq!(task.current_sessions, 0);
        assert_eq!(task.status, Status::Active);
        assert!(!task.default);
    }

    #[test]
    fn test_is_completed() {
        let mut task = create_test_task();
        assert!(!task.is_completed());

        task.current_sessions = 5;
        task.status = Status::Completed;
        assert!(task.is_completed());
    }

    #[test]
    fn test_increment_session() {
        let mut task = create_test_task();

        assert!(task.increment_session().is_ok());
        assert_eq!(task.current_sessions, 1);

        task.current_sessions = 4;
        assert!(task.increment_session().is_ok());
        assert_eq!(task.current_sessions, 5);
        assert_eq!(task.status, Status::Completed);
        assert!(task.completed_at.is_some());

        assert!(task.increment_session().is_err());
    }

    #[test]
    fn test_reset_sessions() {
        let mut task = create_test_task();
        task.current_sessions = 3;
        task.status = Status::Completed;
        task.completed_at = Some(Utc::now());

        task.reset_sessions();

        assert_eq!(task.current_sessions, 0);
        assert_eq!(task.status, Status::Active);
        assert!(task.completed_at.is_none());
    }

    #[test]
    fn test_get_progress_ratio() {
        let mut task = create_test_task();
        assert_eq!(task.get_progress_ratio(), 0.0);

        task.current_sessions = 2;
        assert_eq!(task.get_progress_ratio(), 0.4);

        task.current_sessions = 5;
        assert_eq!(task.get_progress_ratio(), 1.0);

        task.max_sessions = 0;
        assert_eq!(task.get_progress_ratio(), 1.0);
    }

    #[test]
    fn test_get_remaining_sessions() {
        let mut task = create_test_task();
        assert_eq!(task.get_remaining_sessions(), 5);

        task.current_sessions = 2;
        assert_eq!(task.get_remaining_sessions(), 3);

        task.current_sessions = 5;
        assert_eq!(task.get_remaining_sessions(), 0);

        task.current_sessions = 10;
        assert_eq!(task.get_remaining_sessions(), 0);
    }

    #[test]
    fn test_pause() {
        let mut task = create_test_task();

        assert!(task.pause().is_ok());
        assert_eq!(task.status, Status::Paused);

        task.status = Status::Completed;
        assert!(task.pause().is_err());
    }

    #[test]
    fn test_activate() {
        let mut task = create_test_task();
        task.status = Status::Paused;

        assert!(task.activate().is_ok());
        assert_eq!(task.status, Status::Active);

        task.status = Status::Completed;
        assert!(task.activate().is_err());
    }

    #[test]
    fn test_queue() {
        let mut task = create_test_task();

        assert!(task.queue().is_ok());
        assert_eq!(task.status, Status::Queued);

        task.status = Status::Completed;
        assert!(task.queue().is_err());
    }

    #[test]
    fn test_default_management() {
        let mut task = create_test_task();

        assert!(!task.is_default());

        task.set_as_default();
        assert!(task.is_default());

        task.unset_as_default();
        assert!(!task.is_default());

        let task2 = task.clone().with_default(true);
        assert!(task2.is_default());
    }

    #[test]
    fn test_config_management() {
        let mut task = create_test_task();
        let _initial_config = task.get_config().clone();

        let new_config = Config::default();
        task.set_config(new_config.clone());
        assert_eq!(*task.get_config(), new_config);

        task.reset_config();
        assert_eq!(*task.get_config(), Config::default());
    }
}
