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

pub mod config;
pub mod task;
pub mod timer;
pub mod audio;
pub mod events;
pub mod notifications;

// Config infrastructure
pub use config::{
    ConfigRepository, ConfigRepo, FileConfigRepo, ConfigError,
    InMemoryConfigRepository, ConfigBuilder,
    AudioConfigDto, GeneralConfigDto, NotificationConfigDto, AppearanceConfigDto, ConfigDto
};

// Task infrastructure
pub use task::{
    TaskDto, TaskAudioConfigDto, TaskConfigDto,
    FileTaskRepository, InMemoryTaskRepository, TaskRepositoryArc,
    StandardTaskCyclerService
};

// Timer infrastructure
pub use timer::TimerService;

// Audio infrastructure
pub use audio::{RodioAudioService, DefaultAudioAssetProvider, BG_SOUNDS};

// Events infrastructure
pub use events::{
    InMemoryEventBus, EventHandler,
    audio_events,
};

// Notifications infrastructure
pub use notifications::send_phase_notification;