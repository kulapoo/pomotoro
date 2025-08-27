//! Configuration Domain Infrastructure
//!
//! Contains all configuration-related infrastructure implementations:
//! - File-based configuration repository
//! - In-memory configuration repository  
//! - Configuration builder and validation
//! - Configuration DTOs for persistence

pub mod builder;
pub mod config_dto;
pub mod file_repo;
pub mod file_repository_adapter;
pub mod memory_repo;

pub use builder::ConfigBuilder;
pub use config_dto::{
    AppearanceConfigDto, AudioConfigDto, ConfigDto, GeneralConfigDto,
    NotificationConfigDto,
};
pub use file_repo::{
    ConfigError, ConfigRepo, ConfigRepository, FileConfigRepo,
};
pub use file_repository_adapter::FileConfigRepository;
pub use memory_repo::InMemoryConfigRepository;
