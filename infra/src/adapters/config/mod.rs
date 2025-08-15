//! Configuration Domain Infrastructure
//!
//! Contains all configuration-related infrastructure implementations:
//! - File-based configuration repository
//! - In-memory configuration repository  
//! - Configuration builder and validation
//! - Configuration DTOs for persistence

pub mod file_repo;
pub mod memory_repo;
pub mod builder;
pub mod config_dto;

pub use file_repo::{ConfigRepository, ConfigRepo, FileConfigRepo, ConfigError};
pub use memory_repo::InMemoryConfigRepository;
pub use builder::ConfigBuilder;
pub use config_dto::{AudioConfigDto, GeneralConfigDto, NotificationConfigDto, AppearanceConfigDto, ConfigDto};