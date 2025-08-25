use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Timer requires an active entity to start")]
    NoActiveEntity,

    #[error("Invalid timer configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Operation not allowed in current state: {0}")]
    InvalidOperation(String),
}

pub type Result<T> = std::result::Result<T, Error>;
