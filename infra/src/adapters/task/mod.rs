//! Task Domain Infrastructure
//!
//! Contains all task-related infrastructure implementations:
//! - File-based task repository
//! - In-memory task repository
//! - Task cycling service
//! - Task DTOs for persistence
//! - Event handlers for task domain events

pub mod config_dto;
pub mod cycling_srv;
pub mod event_handlers;
pub mod file_repo;
pub mod memory_repo;
pub mod task_dto;

pub use config_dto::TaskConfigDto;
pub use cycling_srv::StandardTaskCyclerService;
pub use event_handlers::{register_task_handlers, unregister_task_handlers};
pub use file_repo::FileTaskRepository;
pub use memory_repo::{InMemoryTaskRepository, TaskRepositoryArc};
pub use task_dto::{TaskAudioConfigDto, TaskDto};
