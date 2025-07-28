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
}

impl From<crate::AudioError> for Error {
    fn from(audio_error: crate::AudioError) -> Self {
        Error::AudioError {
            message: audio_error.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;