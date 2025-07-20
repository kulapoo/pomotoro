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
}

pub type Result<T> = std::result::Result<T, Error>;