//! Task Domain Infrastructure
//!
//! Contains all task-related infrastructure implementations:
//! - File-based task repository
//! - In-memory task repository
//! - Task cycling service
//! - Event handlers for task domain events

pub mod cycling_srv;
pub mod event_handlers;
mod sqlite_repository;
pub use cycling_srv::DefaultCyclingService;
pub use event_handlers::{register_task_handlers, unregister_task_handlers};

pub use sqlite_repository::SqliteTaskRepository;
