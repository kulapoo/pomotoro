//! Application Layer
//!
//! This layer contains the use cases that orchestrate domain services and repositories
//! to implement the business requirements. Use cases follow the Clean Architecture
//! principles by depending only on abstractions (traits) from the domain layer.
//!
//! ## Structure
//!
//! - `task/` - Task management use cases (CRUD, switching, cycling)
//! - `timer/` - Timer session use cases (start, pause, reset, complete)
//! - `config/` - Configuration management use cases (get, update, reset)
//! - `audio/` - Audio management use cases (playback, library, notifications)
//!
//! ## Design Principles
//!
//! - Use cases are independent and can be composed
//! - All dependencies are injected as trait objects
//! - Commands and queries are explicitly defined
//! - Domain events are published after successful operations
//! - Error handling follows domain error types

pub mod task;
pub mod timer;
pub mod config;
pub mod audio;
pub mod events;

mod bootstrap;

pub use bootstrap::bootstrap;

pub use task::*;
pub use timer::{
    start_session, StartSessionCmd, pause_session, resume_session,
    reset_session, reset_full_session, complete_timer_session,
    force_complete_timer_session, SessionCompleted
};
pub use config::*;
pub use audio::*;
pub use events::HandlerRegistry;