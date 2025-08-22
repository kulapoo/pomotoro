use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Task not found: {id}")]
    TaskNotFound { id: String },

    #[error("Invalid timer duration: {duration} seconds")]
    InvalidDuration { duration: u32 },

    #[error("Invalid session count: {count}")]
    InvalidSessionCount { count: u8 },

    #[error("Task already completed")]
    TaskAlreadyCompleted,

    #[error("Invalid task lifecycle: {message}")]
    InvalidLifecycle { message: String },

    #[error("Default task not found")]
    DefaultTaskNotFound,

    #[error("Timer state transition not allowed: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Invalid tag format: {tag}")]
    InvalidTagFormat { tag: String },

    #[error("Tag too long: {tag} (max length: {max_length})")]
    TagTooLong { tag: String, max_length: usize },

    #[error("Invalid task name: cannot be empty")]
    EmptyTaskName,

    #[error("Invalid volume: {volume} (must be between 0.0 and 1.0)")]
    InvalidVolume { volume: f32 },

    #[error("Repository error: {message}")]
    RepositoryError { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Audio error: {message}")]
    AudioError { message: String },

    #[error("Event publishing error: {message}")]
    EventPublishingError { message: String },
}

impl From<crate::AudioError> for Error {
    fn from(audio_error: crate::AudioError) -> Self {
        Error::AudioError {
            message: audio_error.to_string(),
        }
    }
}

impl From<crate::timer::Error> for Error {
    fn from(timer_error: crate::timer::Error) -> Self {
        match timer_error {
            crate::timer::Error::InvalidStateTransition { from, to } => {
                Error::InvalidStateTransition { from, to }
            }
            crate::timer::Error::NoActiveEntity => {
                Error::ConfigurationError {
                    message: "Timer requires an active entity to start".to_string(),
                }
            }
            crate::timer::Error::InvalidConfiguration(msg) => {
                Error::ConfigurationError { message: msg }
            }
            crate::timer::Error::InvalidOperation(msg) => {
                Error::ConfigurationError { message: msg }
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;