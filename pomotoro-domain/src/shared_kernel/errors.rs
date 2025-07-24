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

    #[error("Repository error: {message}")]
    RepositoryError { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Audio error: {message}")]
    AudioError { message: String },
}

pub type Result<T> = std::result::Result<T, Error>;