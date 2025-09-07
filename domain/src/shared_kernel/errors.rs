use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    // Task domain errors
    #[error("Task not found: {id}")]
    TaskNotFound { id: String },

    #[error("Invalid task params: {message}")]
    InvalidTaskParams { message: String },

    #[error("Task creation error: {message}")]
    TaskCreationError { message: String },

    #[error("Task already completed")]
    TaskAlreadyCompleted,

    #[error("Invalid task lifecycle: {message}")]
    InvalidLifecycle { message: String },

    #[error("Default task not found")]
    DefaultTaskNotFound,

    #[error("Invalid task name: cannot be empty")]
    EmptyTaskName,

    // Timer domain errors
    #[error("Invalid timer duration: {duration} seconds")]
    InvalidDuration { duration: u32 },

    #[error("Invalid session count: {count}")]
    InvalidSessionCount { count: u8 },

    #[error("Timer state transition not allowed: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },

    // Tag domain errors
    #[error("Invalid tag format: {tag}")]
    InvalidTagFormat { tag: String },

    #[error("Tag too long: {tag} (max length: {max_length})")]
    TagTooLong { tag: String, max_length: usize },

    // Audio domain errors
    #[error("Invalid volume: {volume} (must be between 0.0 and 1.0)")]
    InvalidVolume { volume: f32 },

    #[error("Audio error: {message}")]
    AudioError { message: String },

    // Infrastructure errors
    #[error("Repository error: {message}")]
    RepositoryError { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Event publishing error: {message}")]
    EventPublishingError { message: String },

    #[error("Event handling error: {message}")]
    EventHandlingError { message: String },

    #[error("Serialization error: {message}")]
    SerializationError { message: String },

    #[error("Deserialization error: {message}")]
    DeserializationError { message: String },

    #[error("IO error: {message}")]
    IoError { message: String },
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
            crate::timer::Error::NoActiveEntity => Error::ConfigurationError {
                message: "Timer requires an active entity to start".to_string(),
            },
            crate::timer::Error::InvalidConfiguration(msg) => {
                Error::ConfigurationError { message: msg }
            }
            crate::timer::Error::InvalidOperation(msg) => {
                Error::ConfigurationError { message: msg }
            }
            crate::timer::Error::RepositoryError { message } => {
                Error::ConfigurationError { message }
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_messages() {
        let error = Error::TaskNotFound {
            id: "task-123".to_string(),
        };
        assert_eq!(error.to_string(), "Task not found: task-123");

        let error = Error::InvalidDuration { duration: 0 };
        assert_eq!(error.to_string(), "Invalid timer duration: 0 seconds");

        let error = Error::TaskAlreadyCompleted;
        assert_eq!(error.to_string(), "Task already completed");

        let error = Error::EmptyTaskName;
        assert_eq!(error.to_string(), "Invalid task name: cannot be empty");
    }

    #[test]
    fn test_error_with_parameters() {
        let error = Error::InvalidSessionCount { count: 255 };
        assert_eq!(error.to_string(), "Invalid session count: 255");

        let error = Error::InvalidVolume { volume: 1.5 };
        assert_eq!(
            error.to_string(),
            "Invalid volume: 1.5 (must be between 0.0 and 1.0)"
        );

        let error = Error::TagTooLong {
            tag: "very_long_tag".to_string(),
            max_length: 10,
        };
        assert_eq!(
            error.to_string(),
            "Tag too long: very_long_tag (max length: 10)"
        );
    }

    #[test]
    fn test_error_with_messages() {
        let error = Error::TaskCreationError {
            message: "Invalid parameters".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Task creation error: Invalid parameters"
        );

        let error = Error::InvalidLifecycle {
            message: "Cannot transition from completed state".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Invalid task lifecycle: Cannot transition from completed state"
        );

        let error = Error::RepositoryError {
            message: "Database connection failed".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Repository error: Database connection failed"
        );
    }

    #[test]
    fn test_state_transition_error() {
        let error = Error::InvalidStateTransition {
            from: "Idle".to_string(),
            to: "Paused".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Timer state transition not allowed: Idle -> Paused"
        );
    }

    #[test]
    fn test_serialization_errors() {
        let error = Error::SerializationError {
            message: "Failed to serialize object".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Serialization error: Failed to serialize object"
        );

        let error = Error::DeserializationError {
            message: "Invalid JSON format".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Deserialization error: Invalid JSON format"
        );
    }

    #[test]
    fn test_error_clone() {
        let original = Error::TaskNotFound {
            id: "original-id".to_string(),
        };
        let cloned = original.clone();
        assert_eq!(original.to_string(), cloned.to_string());
    }

    #[test]
    fn test_result_type() {
        let success: Result<i32> = Ok(42);
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), 42);

        let failure: Result<i32> = Err(Error::DefaultTaskNotFound);
        assert!(failure.is_err());
        assert_eq!(failure.unwrap_err().to_string(), "Default task not found");
    }
}
