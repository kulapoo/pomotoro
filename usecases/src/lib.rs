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

pub mod audio;
pub mod config;
pub mod task;
pub mod timer;

pub use audio::*;
pub use config::*;
pub use task::*;
