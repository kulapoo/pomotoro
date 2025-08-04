//! Task Domain Infrastructure
//!
//! Contains all task-related infrastructure implementations:
//! - File-based task repository
//! - In-memory task repository
//! - Task cycling service
//! - Task DTOs for persistence

pub mod dto;
pub mod config_dto;
pub mod file_repo;
pub mod memory_repo;
pub mod cycling_service;

pub use dto::{TaskDto, TaskAudioConfigDto};
pub use config_dto::TaskConfigDto;
pub use file_repo::FileTaskRepository;
pub use memory_repo::{InMemoryTaskRepository, TaskRepositoryArc};
pub use cycling_service::StandardTaskCyclerService;