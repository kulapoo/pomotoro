//! Task Domain Infrastructure
//!
//! Contains all task-related infrastructure implementations:
//! - File-based task repository
//! - In-memory task repository
//! - Task cycling service
//! - Task DTOs for persistence
//! - Event handlers for task domain events

pub mod common;
pub mod cycling_srv;
pub mod event_handlers;
pub mod task_dto;

pub use common::TaskRepositoryArc;
pub use cycling_srv::DefaultCyclingService;
pub use event_handlers::{register_task_handlers, unregister_task_handlers};
pub use task_dto::TaskDto;
