use super::{config::Config, id::Id, status::Status};
use crate::{AudioConfig, Error, Result, TaskDefaults};
use chrono::{DateTime, Utc};
use std::time::Duration;

// Default values for Builder - autonomous construction without external dependencies
const DEFAULT_WORK_DURATION: Duration = Duration::from_secs(25 * 60);
const DEFAULT_SHORT_BREAK_DURATION: Duration = Duration::from_secs(5 * 60);
const DEFAULT_LONG_BREAK_DURATION: Duration = Duration::from_secs(15 * 60);
const DEFAULT_SESSIONS_UNTIL_LONG_BREAK: u8 = 4;
const DEFAULT_ENABLE_SCREEN_BLOCKING: bool = false;
const DEFAULT_MAX_SESSIONS: u8 = 4;

/// Builder for constructing Task instances with fluent interface and centralized validation
#[derive(Debug, Clone)]
pub struct Builder {
    id: Option<Id>,
    name: Option<String>,
    description: Option<String>,
    max_sessions: Option<u8>,
    current_sessions: Option<u8>,
    tags: Option<Vec<String>>,
    config: Option<Config>,
    audio_config: Option<AudioConfig>,
    created_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    status: Option<Status>,
    default: Option<bool>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    /// Create a new Builder with all fields unset
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            description: None,
            max_sessions: None,
            current_sessions: None,
            tags: None,
            config: None,
            audio_config: None,
            created_at: None,
            completed_at: None,
            status: None,
            default: None,
        }
    }

    /// Create a builder with name and max_sessions set (common pattern)
    pub fn with_name_and_sessions(name: String, max_sessions: u8) -> Self {
        Self::new().name(name).max_sessions(max_sessions)
    }

    /// Create a builder for a default task
    pub fn default_task() -> Self {
        Self::new()
            .name("Focus Session".to_string())
            .description("Default pomodoro task for focused work".to_string())
            .tags(vec!["focus".to_string()])
            .status(Status::Active)
            .default(true)
    }

    /// Set the task ID
    pub fn id(mut self, id: Id) -> Self {
        self.id = Some(id);
        self
    }

    /// Set the task name
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set the task description
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set the maximum number of sessions
    pub fn max_sessions(mut self, max_sessions: u8) -> Self {
        self.max_sessions = Some(max_sessions);
        self
    }

    /// Set the current session count
    pub fn current_sessions(mut self, current_sessions: u8) -> Self {
        self.current_sessions = Some(current_sessions);
        self
    }

    /// Set the task tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Set the task configuration
    pub fn config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Set the audio configuration
    pub fn audio_config(mut self, audio_config: AudioConfig) -> Self {
        self.audio_config = Some(audio_config);
        self
    }

    /// Set description (builder method for fluent API)
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set tags (builder method for fluent API)
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Set configuration with validation (builder method for fluent API)
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Set audio configuration with validation (builder method for fluent API)
    pub fn with_audio_config(mut self, audio_config: AudioConfig) -> Self {
        self.audio_config = Some(audio_config);
        self
    }

    /// Set the creation timestamp
    pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = Some(created_at);
        self
    }

    /// Set the completion timestamp
    pub fn completed_at(mut self, completed_at: DateTime<Utc>) -> Self {
        self.completed_at = Some(completed_at);
        self
    }

    /// Set the task status
    pub fn status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }

    /// Set the default flag
    pub fn default(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }

    /// Mark the task as completed
    pub fn completed(self) -> Self {
        self.status(Status::Completed).completed_at(Utc::now())
    }

    /// Build the Task with centralized validation using built-in defaults
    pub fn build(self) -> Result<super::Task> {
        // Validate required fields
        let name = self.name.ok_or(Error::EmptyTaskName)?;
        if name.trim().is_empty() {
            return Err(Error::EmptyTaskName);
        }

        let max_sessions = self.max_sessions.unwrap_or(DEFAULT_MAX_SESSIONS);
        if max_sessions == 0 {
            return Err(Error::InvalidSessionCount {
                count: max_sessions,
            });
        }

        let current_sessions = self.current_sessions.unwrap_or(0);
        if current_sessions > max_sessions {
            return Err(Error::InvalidSessionCount {
                count: current_sessions,
            });
        }

        // Create or use provided config
        let config = match self.config {
            Some(config) => config,
            None => Config::new(
                DEFAULT_WORK_DURATION,
                DEFAULT_SHORT_BREAK_DURATION,
                DEFAULT_LONG_BREAK_DURATION,
                DEFAULT_SESSIONS_UNTIL_LONG_BREAK,
                DEFAULT_ENABLE_SCREEN_BLOCKING,
            )?,
        };

        // Validate audio config if provided
        let audio_config = self.audio_config.unwrap_or_default();
        audio_config.validate()?;

        // Determine status based on completion state
        let status = self.status.unwrap_or({
            if current_sessions >= max_sessions {
                Status::Completed
            } else {
                Status::Queued
            }
        });

        // Set completion time if completed
        let completed_at = if status == Status::Completed {
            self.completed_at.or_else(|| Some(Utc::now()))
        } else {
            None
        };

        Ok(super::Task {
            id: self.id.unwrap_or_default(),
            name: name.trim().to_string(),
            description: self.description,
            max_sessions,
            current_sessions,
            tags: self.tags.unwrap_or_default(),
            config,
            audio_config,
            created_at: self.created_at.unwrap_or_else(Utc::now),
            completed_at,
            status,
            default: self.default.unwrap_or(false),
        })
    }

    /// Build the Task with custom defaults (for configuration management)
    pub fn build_with_defaults(
        self,
        defaults: &TaskDefaults,
    ) -> Result<super::Task> {
        // Validate required fields
        let name = self.name.ok_or(Error::EmptyTaskName)?;
        if name.trim().is_empty() {
            return Err(Error::EmptyTaskName);
        }

        let max_sessions =
            self.max_sessions.unwrap_or(defaults.max_sessions_default);
        if max_sessions == 0 {
            return Err(Error::InvalidSessionCount {
                count: max_sessions,
            });
        }

        let current_sessions = self.current_sessions.unwrap_or(0);
        if current_sessions > max_sessions {
            return Err(Error::InvalidSessionCount {
                count: current_sessions,
            });
        }

        // Create or use provided config
        let config = match self.config {
            Some(config) => config,
            None => Config::new(
                defaults.work_duration,
                defaults.short_break_duration,
                defaults.long_break_duration,
                defaults.sessions_until_long_break,
                defaults.enable_screen_blocking,
            )?,
        };

        // Validate audio config if provided
        let audio_config = self.audio_config.unwrap_or_default();
        audio_config.validate()?;

        // Determine status based on completion state
        let status = self.status.unwrap_or({
            if current_sessions >= max_sessions {
                Status::Completed
            } else {
                Status::Queued
            }
        });

        // Set completion time if completed
        let completed_at = if status == Status::Completed {
            self.completed_at.or_else(|| Some(Utc::now()))
        } else {
            None
        };

        Ok(super::Task {
            id: self.id.unwrap_or_default(),
            name: name.trim().to_string(),
            description: self.description,
            max_sessions,
            current_sessions,
            tags: self.tags.unwrap_or_default(),
            config,
            audio_config,
            created_at: self.created_at.unwrap_or_else(Utc::now),
            completed_at,
            status,
            default: self.default.unwrap_or(false),
        })
    }
}
