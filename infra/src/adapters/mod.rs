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
//! ## Domain-Based Structure
//!
//! - `config/`: Configuration domain infrastructure
//! - `task/`: Task domain infrastructure
//! - `timer/`: Timer domain infrastructure
//! - `audio/`: Audio domain infrastructure
//! - `events/`: Event publishing infrastructure
//! - `notifications`: System notification integration
//! - `storage/`: File storage management

pub mod audio;
pub mod database;
pub mod events;
pub mod notifications;
pub mod task;
pub mod timer;

// Task infrastructure
pub use task::{
    TaskDto, TaskRepositoryArc,
};

// Timer infrastructure
pub use timer::SqliteTimerService;

// Audio infrastructure
pub use audio::{BG_SOUNDS, DefaultAudioAssetProvider, RodioAudioService};

// Events infrastructure
pub use events::{EventHandler, InMemoryEventBus, audio_events};

// Notifications infrastructure
pub use notifications::{NotificationService, register_notification_handlers};

// Database infrastructure
pub use database::{DbPool, establish_connection, run_migrations, SqliteTaskRepository, SqliteConfigRepository, SqliteTimerRepository, TimerRepository};
