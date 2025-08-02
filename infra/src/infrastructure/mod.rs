//! Infrastructure Layer
//!
//! This layer contains concrete implementations of domain interfaces and handles
//! external concerns like persistence, event publishing, notifications, and 
//! integration with external systems.
//!
//! ## Clean Architecture Principles
//!
//! - **Implementation**: Concrete implementations of domain abstractions
//! - **External Concerns**: Database, file system, network, UI frameworks
//! - **Dependency Direction**: Depends on domain abstractions, not implementations
//! - **Isolation**: Changes here shouldn't affect domain logic
//!
//! ## Structure
//!
//! - `task`: Task domain infrastructure with repository implementation
//! - `config`: Configuration domain infrastructure with repository implementation
//! - `timer`: Timer domain infrastructure with service implementation
//! - `audio`: Audio domain infrastructure with service implementation
//! - `events`: Event publishing infrastructure
//! - `notifications`: System notification integration
//! - `config_builder`: Configuration infrastructure models

pub mod config_repo;
pub mod timer_srv;
pub mod audio_srv;
pub mod audio_asset_provider;
pub mod events;
pub mod notifications;
pub mod config_builder;
pub mod timer_models;
pub mod repositories;
pub mod persistence;
pub mod task_cycling_srv;

pub use config_repo::*;
pub use timer_srv::*;
pub use audio_srv::RodioAudioService;
pub use audio_asset_provider::*;
pub use events::{EventPublisherArc, create_composite_event_publisher, create_event_publisher_with_bus, DomainEventBus};
pub use notifications::*;
pub use config_builder::*;
// timer_models is now handled by domain layer
pub use repositories::*;
pub use persistence::*;
pub use task_cycling_srv::StandardTaskCyclerService;